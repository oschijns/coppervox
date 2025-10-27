extends Node


@export
var inputs := PackedInt32Array([0x10, 0x30, 0x20, 0x40])

@export
var operations := PackedInt32Array([0, 1, 4, 5])

@export
var connections := PackedInt32Array([0b11, 0b101, 0b110, 0b1010])


var _output_buffer : RID


# Flag for storage buffers used in read-write operations
const RW_BUFFER := RenderingDevice.BUFFER_CREATION_AS_STORAGE_BIT


func _ready() -> void:

	# Create a local rendering device
	var rd := RenderingServer.create_local_rendering_device()

	# Load GLSL shader
	var shader_file : RDShaderFile = load("res://gpu/process.glsl")
	var shader   := rd.shader_create_from_spirv(shader_file.get_spirv())
	var pipeline := rd.compute_pipeline_create(shader)

	# Create uniforms to assign the buffer to the rendering device
	var uni_size := create_sizes      (rd)
	var uni_conn := create_connections(rd)
	var uni_op   := create_operations (rd)
	var uni_in   := create_inputs     (rd)
	var uni_out  := create_outputs    (rd)

	# Set uniforms
	var uniform_set := rd.uniform_set_create([uni_size, uni_conn, uni_op, uni_in, uni_out], shader, 0)

	# Create a compute pipeline
	var compute_list := rd.compute_list_begin()
	rd.compute_list_bind_compute_pipeline(compute_list, pipeline)
	rd.compute_list_bind_uniform_set(compute_list, uniform_set, 0)
	rd.compute_list_dispatch(compute_list, 256, 1, 1)
	rd.compute_list_end()

	# Submit to GPU and wait for sync
	rd.submit()
	rd.sync()

	# Read back the data from the buffer
	var result := rd.buffer_get_data(_output_buffer).to_int32_array()
	print("Output: ", result)



func create_sizes(rd: RenderingDevice) -> RDUniform:
	var size    := inputs.size()
	var data    := PackedInt32Array([size, size])
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


func create_inputs(rd: RenderingDevice) -> RDUniform:
	var bytes   := inputs.to_byte_array()
	var buffer  := rd.storage_buffer_create(bytes.size(), bytes, 0, RW_BUFFER)
	var uniform := RDUniform.new()

	uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uniform.binding = 3
	uniform.add_id(buffer)

	return uniform


func create_outputs(rd: RenderingDevice) -> RDUniform:
	var size := inputs.size() * 4
	_output_buffer = rd.storage_buffer_create(size, PackedByteArray(), 0, RW_BUFFER)
	var uniform := RDUniform.new()

	uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uniform.binding = 4
	uniform.add_id(_output_buffer)

	return uniform



func _backup() -> void:

	# Create a local rendering device
	var rd := RenderingServer.create_local_rendering_device()

	# Load GLSL shader
	var shader_file : RDShaderFile = load("res://gpu/process.glsl")
	var shader   := rd.shader_create_from_spirv(shader_file.get_spirv())
	var pipeline := rd.compute_pipeline_create(shader)

	# Prepare static buffers
	var data_size := PackedInt32Array([4, 4])
	var data_conn := PackedInt32Array([0b1111, 0b1111, 0b1111, 0b1111])
	var data_op   := PackedInt32Array([0, 1, 2, 3])

	var bytes_size := data_size.to_byte_array()
	var bytes_conn := data_conn.to_byte_array()
	var bytes_op   := data_op  .to_byte_array()

	var buf_size := rd.storage_buffer_create(bytes_size.size(), bytes_size)
	var buf_conn := rd.storage_buffer_create(bytes_conn.size(), bytes_conn)
	var buf_op   := rd.storage_buffer_create(bytes_op  .size(), bytes_op  )

	# Prepare the two dynamic buffers to process
	var size_buf := 4 * 4
	var buf_a := rd.storage_buffer_create(size_buf, PackedByteArray(), 0, RW_BUFFER)
	var buf_b := rd.storage_buffer_create(size_buf, PackedByteArray(), 0, RW_BUFFER)

	# Create uniforms to assign the buffer to the rendering device
	var uni_size := RDUniform.new()
	uni_size.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uni_size.binding = 0
	uni_size.add_id(buf_size)

	var uni_conn := RDUniform.new()
	uni_conn.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uni_conn.binding = 1
	uni_conn.add_id(buf_conn)

	var uni_op := RDUniform.new()
	uni_op.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uni_op.binding = 2
	uni_op.add_id(buf_op)

	var uni_in := RDUniform.new()
	uni_in.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uni_in.binding = 3
	uni_in.add_id(buf_a)

	var uni_out := RDUniform.new()
	uni_out.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uni_out.binding = 4
	uni_out.add_id(buf_b)

	# Set uniforms
	var uniform_set := rd.uniform_set_create([uni_size, uni_conn, uni_op, uni_in, uni_out], shader, 0)

	# Create a compute pipeline
	var compute_list := rd.compute_list_begin()
	rd.compute_list_bind_compute_pipeline(compute_list, pipeline)
	rd.compute_list_bind_uniform_set(compute_list, uniform_set, 0)
	rd.compute_list_dispatch(compute_list, 256, 1, 1)
	rd.compute_list_end()

	# Submit to GPU and wait for sync
	rd.submit()
	rd.sync()

	# Read back the data from the buffer
	var result := rd.buffer_get_data(buf_b).to_int32_array()
	print("Output: ", result)