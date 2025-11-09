extends Node

# workgroup sized as defined in the compute shader
const WORKGROUP := 256

# Operations as defined in the compute shader
enum Op {
	OR   = 0,
	AND  = 1,
	NOR  = 2,
	NAND = 3,
	ADD  = 4,
	MUL  = 5,
	MIN  = 6,
	MAX  = 7
}

@export
var inputs := PackedInt32Array([0x0, 0x13, 0x23, 0x43])

@export
var operations := PackedInt32Array([Op.OR, Op.AND, Op.ADD, Op.MAX])

@export
var connections := PackedInt32Array([0b1110, 0b1110, 0b1110, 0b1110])


# How many dispatch to generate
@onready
var _dispatch := ceili(float(inputs.size()) / WORKGROUP)


# Pipeline created for running the compute shader
var _pipeline := RID()


# Handle the two buffers to swap
var _buffer_in  := RID()
var _buffer_out := RID()


# Flag for storage buffers used in read-write operations
const RW_BUFFER := RenderingDevice.BUFFER_CREATION_AS_STORAGE_BIT

# Flag for output texture
const TEXTURE_BUFFER := (0
	| RenderingDevice.TEXTURE_USAGE_SAMPLING_BIT
	| RenderingDevice.TEXTURE_USAGE_STORAGE_BIT
)


func _ready() -> void:

	# Create a local rendering device
	var rd := RenderingServer.create_local_rendering_device()

	# Load GLSL shader
	var shader_file : RDShaderFile = load("res://gpu/process.glsl")
	var shader   := rd.shader_create_from_spirv(shader_file.get_spirv())
	_pipeline = rd.compute_pipeline_create(shader)

	# Create uniforms to assign the buffer to the rendering device
	var uni_size := create_sizes      (rd)
	var uni_conn := create_connections(rd)
	var uni_op   := create_operations (rd)
	var uni_in   := create_inputs     (rd)
	var uni_out  := create_outputs    (rd, inputs.size())
	var uni_tex  := create_texture_rd (rd)

	# Set uniforms
	var uniform_set : Array[RDUniform] = [uni_size, uni_conn, uni_op, uni_in, uni_out, uni_tex] 

	run_compute(rd, shader, uniform_set)
	run_compute(rd, shader, uniform_set)
	run_compute(rd, shader, uniform_set)
	run_compute(rd, shader, uniform_set)


func run_compute(rd: RenderingDevice, shader: RID, uniform_set: Array[RDUniform]) -> void:
	var uniforms := rd.uniform_set_create(uniform_set, shader, 0)
	
	# Create a compute pipeline
	var compute_list := rd.compute_list_begin()
	rd.compute_list_bind_compute_pipeline(compute_list, _pipeline)
	rd.compute_list_bind_uniform_set(compute_list, uniforms, 0)
	rd.compute_list_dispatch(compute_list, _dispatch, 1, 1)
	rd.compute_list_end()

	# Submit to GPU and wait for sync
	rd.submit()
	rd.sync()

	# Read back the data from the buffer
	var result := rd.buffer_get_data(_buffer_out).to_int32_array()
	var output := ""
	for i in range(result.size()):
		output += " %04x," % result[i]
	print(output)



func create_sizes(rd: RenderingDevice) -> RDUniform:
	var bytes   := PackedByteArray()
	bytes.resize(2 * 8)
	bytes.encode_u64(0, inputs.size())
	bytes.encode_u64(4, operations.size())
	var buffer  := rd.uniform_buffer_create(bytes.size(), bytes)
	var uniform := RDUniform.new()

	uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_UNIFORM_BUFFER
	uniform.binding = 0
	uniform.add_id(buffer)

	return uniform


func create_connections(rd: RenderingDevice) -> RDUniform:
	var bytes   := connections.to_byte_array()
	var buffer  := rd.storage_buffer_create(bytes.size(), bytes)
	var uniform := RDUniform.new()

	uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uniform.binding = 1
	uniform.add_id(buffer)

	return uniform


func create_operations(rd: RenderingDevice) -> RDUniform:
	var bytes   := operations.to_byte_array()
	var buffer  := rd.storage_buffer_create(bytes.size(), bytes)
	var uniform := RDUniform.new()

	uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uniform.binding = 2
	uniform.add_id(buffer)

	return uniform


func create_inputs(rd: RenderingDevice) -> RDUniform:
	var bytes   := inputs.to_byte_array()
	_buffer_in   = rd.storage_buffer_create(bytes.size(), bytes, 0, RW_BUFFER)
	var uniform := RDUniform.new()

	uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uniform.binding = 3
	uniform.add_id(_buffer_in)

	return uniform


func create_outputs(rd: RenderingDevice, size: int) -> RDUniform:
	var bytes   := PackedByteArray()
	bytes.resize(operations.size() * 4)
	_buffer_out  = rd.storage_buffer_create(bytes.size(), bytes, 0, RW_BUFFER)
	var uniform := RDUniform.new()

	uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uniform.binding = 4
	uniform.add_id(_buffer_out)

	return uniform


func create_texture_rd(rd: RenderingDevice) -> RDUniform:
	var fmt := RDTextureFormat.new()
	fmt.width        = inputs.size();
	fmt.height       = 1
	fmt.depth        = 1
	fmt.format       = RenderingDevice.DATA_FORMAT_R32_UINT
	fmt.samples      = RenderingDevice.TEXTURE_SAMPLES_1
	fmt.texture_type = RenderingDevice.TEXTURE_TYPE_1D
	fmt.usage_bits   = TEXTURE_BUFFER
	var view := RDTextureView.new()
	var texture := rd.texture_create(fmt, view)

	var uniform := RDUniform.new()
	uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_IMAGE
	uniform.binding = 5
	uniform.add_id(texture)

	return uniform
