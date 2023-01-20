use core::slice::SliceIndex;

unsafe impl SliceIndex<[u8]> for u16 {
    type Output = [u8];
    fn get(self, slice: &[u8]) -> Option<&Self::Output> {
        
    }
    fn get_mut(self, slice: &mut [u8]) -> Option<&mut Self::Output> {
        
    }
    unsafe fn get_unchecked(self, slice: *const [u8]) -> *const Self::Output {
        
    }
    unsafe fn get_unchecked_mut(self, slice: *mut [u8]) -> *mut Self::Output {
        
    }
}
impl SliceIndex<[u8]> for u8 {
    
}