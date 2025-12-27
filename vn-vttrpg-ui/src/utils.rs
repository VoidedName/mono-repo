use crate::ElementSize;

pub trait ToArray {
    type ArrayType;
    fn to_array(self) -> Self::ArrayType;
}

impl ToArray for (f32, f32) {
    type ArrayType = [f32; 2];

    fn to_array(self) -> Self::ArrayType {
        [self.0, self.1]
    }
}

impl ToArray for ElementSize {
    type ArrayType = [f32; 2];
    fn to_array(self) -> Self::ArrayType {
        [self.width, self.height]
    }
}
