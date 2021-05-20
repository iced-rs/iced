// webgpu requires that a struct be aligned to a multiple of its largest member
pub const fn align_size(size: u64, alignment: u64) -> u64 {
    ((size + alignment - 1) / alignment) * alignment
}
