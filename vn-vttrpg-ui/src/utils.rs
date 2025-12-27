use crate::ConcreteSize;

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

impl ToArray for ConcreteSize {
    type ArrayType = [f32; 2];
    fn to_array(self) -> Self::ArrayType {
        [self.width, self.height]
    }
}