use cubecl::{prelude::*, server::Handle};
use std::marker::PhantomData;

/// Simple GpuTensor
#[derive(Debug, Clone)]
pub struct GpuTensor<R, F>
where
    R: Runtime,
    F: Float + CubeElement,
{
    handle: Handle,
    strides: [usize; 1],
    shape: [usize; 1],
    _r: PhantomData<R>,
    _f: PhantomData<F>,
}

impl<R, F> GpuTensor<R, F>
where
    R: Runtime,
    F: Float + CubeElement,
{
    /// Create a new buffer with the given size
    pub fn new(client: &ComputeClient<R::Server>, shape: usize) -> Self {
        let handle = client.empty(shape * size_of::<F>());
        Self {
            handle,
            strides: [1],
            shape: [shape],
            _r: PhantomData,
            _f: PhantomData,
        }
    }

    /// Create a tensor argument to pass to the kernel
    pub fn into_tensor_arg(&self, factor: u8) -> TensorArg<'_, R> {
        unsafe { TensorArg::from_raw_parts::<F>(&self.handle, &self.strides, &self.shape, factor) }
    }

    /// Read the data computed
    pub fn read(self, client: &ComputeClient<R::Server>) -> Vec<F> {
        let bytes = client.read_one(self.handle);
        F::from_bytes(&bytes).to_vec()
    }
}

/*
#[cube(launch_unchecked)]
fn reduce_matrix<F>(input: &Tensor<F>, output: &mut Tensor<F>)
where
    F: Float + CubeElement,
{
    for i in 0..input.shape(0) {
        unsafe { output.index_assign_unchecked(i, *input.index_unchecked(i)) };
    }
}
*/
