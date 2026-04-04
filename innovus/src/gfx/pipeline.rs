pub trait BindGroup {
    const ENTRIES: &'static [wgpu::BindGroupLayoutEntry];

    fn create_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: Self::ENTRIES,
        })
    }

    fn bind_group(&self) -> &wgpu::BindGroup;
}
