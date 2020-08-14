use crate::util::destroy;

pub struct SourceMap {
    pub data: Vec<u32>,
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_source_map_get_data(
    source_map: *const SourceMap,
    data: *mut *const u32,
    data_len: *mut usize,
) {
    let sm_data = &(*source_map).data;
    *data = sm_data.as_ptr();
    *data_len = sm_data.len();
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_source_map_destroy(source_map: *mut SourceMap) {
    destroy(source_map);
}
