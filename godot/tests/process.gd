extends Node

const WORKGROUP := 256

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
var _buffer_a := RID()
var _buffer_b := RID()

# Create uniforms for the buffers
var _uniform_a_in  : RDUniform = null
var _uniform_a_out : RDUniform = null
var _uniform_b_in  : RDUniform = null
var _uniform_b_out : RDUniform = null


# Flag for storage buffers used in read-write operations
const RW_BUFFER := RenderingDevice.BUFFER_CREATION_AS_STORAGE_BIT


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
	create_swap_buffers(rd)

	# Set uniforms
	var uniform_set_a := rd.uniform_set_create([uni_size, uni_conn, uni_op, _uniform_a_in, _uniform_b_out], shader, 0)
	var uniform_set_b := rd.uniform_set_create([uni_size, uni_conn, uni_op, _uniform_b_in, _uniform_a_out], shader, 0)

	run_compute(rd, uniform_set_a, _buffer_b)
	run_compute(rd, uniform_set_b, _buffer_a)
	run_compute(rd, uniform_set_a, _buffer_b)
	run_compute(rd, uniform_set_b, _buffer_a)


func run_compute(rd: RenderingDevice, uniform_set: RID, buffer: RID) -> void:
	# Create a compute pipeline
	var compute_list := rd.compute_list_begin()
	rd.compute_list_bind_compute_pipeline(compute_list, _pipeline)
	rd.compute_list_bind_uniform_set(compute_list, uniform_set, 0)
	rd.compute_list_dispatch(compute_list, _dispatch, 1, 1)
	rd.compute_list_end()

	# Submit to GPU and wait for sync
	rd.submit()
	rd.sync()

	# Read back the data from the buffer
	var result := rd.buffer_get_data(buffer).to_int32_array()
	var output := ""
	for i in range(result.size()):
		output += " %04x," % result[i]
	print(output)



func create_sizes(rd: RenderingDevice) -> RDUniform:
	var data    := PackedInt32Array([inputs.size()])
	var bytes   := data.to_byte_array()
	var buffer  := rd.storage_buffer_create(bytes.size(), bytes)
	var uniform := RDUniform.new()

	uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
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


func create_swap_buffers(rd: RenderingDevice) -> void:
	var bytes := inputs.to_byte_array()
	_buffer_a = rd.storage_buffer_create(bytes.size(), bytes, 0, RW_BUFFER)
	_buffer_b = rd.storage_buffer_create(bytes.size(), bytes, 0, RW_BUFFER)
	_uniform_a_in  = RDUniform.new()
	_uniform_a_out = RDUniform.new()
	_uniform_b_in  = RDUniform.new()
	_uniform_b_out = RDUniform.new()

	_uniform_a_in.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	_uniform_a_in.binding = 3
	_uniform_a_in.add_id(_buffer_a)

	_uniform_a_out.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	_uniform_a_out.binding = 4
	_uniform_a_out.add_id(_buffer_a)

	_uniform_b_in.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	_uniform_b_in.binding = 3
	_uniform_b_in.add_id(_buffer_b)

	_uniform_b_out.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	_uniform_b_out.binding = 4
	_uniform_b_out.add_id(_buffer_b)
