use std::ops::{Add, AddAssign, Sub, SubAssign};

macro_rules! fixed_vector {
    ($vec:ident, $($i:ident),+) => {
        impl<T: Default> Default for $vec<T> {
            fn default() -> Self {
                Self { $($i: Default::default()),* }
            }
        }

        impl<T: Add<Output=T>> Add for $vec<T> {
            type Output = $vec<T>;
            fn add(self, rhs: Self) -> Self::Output {
                Self { $($i: self.$i + rhs.$i),* }
            }
        }
        impl<T: Sub<Output=T>> Sub for $vec<T> {
            type Output = $vec<T>;
            fn sub(self, rhs: Self) -> Self::Output {
                Self { $($i: self.$i - rhs.$i),* }
            }
        }
        impl<T: AddAssign> AddAssign for $vec<T> {
            fn add_assign(&mut self, rhs: Self) {
                $(self.$i += rhs.$i);*
            }
        }
        impl<T: SubAssign> SubAssign for $vec<T> {
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$i -= rhs.$i);*
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}



#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vec4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

fixed_vector!(Vec2, x, y);
fixed_vector!(Vec3, x, y, z);
fixed_vector!(Vec4, x, y, z, w);

pub const fn vec2<T>(x: T, y: T) -> Vec2<T> {
    Vec2 { x, y }
}
pub const fn vec3<T>(x: T, y: T, z: T) -> Vec3<T> {
    Vec3 { x, y, z }
}
pub const fn vec4<T>(x: T, y: T, z: T, w: T) -> Vec4<T> {
    Vec4 { x, y, z, w }
}

macro_rules! derive_vectors {
    ($($T:ty),+) => {
        $(
            unsafe impl bytemuck::Zeroable for Vec2<$T> {}
            unsafe impl bytemuck::Zeroable for Vec3<$T> {}
            unsafe impl bytemuck::Zeroable for Vec4<$T> {}
    
            unsafe impl bytemuck::Pod for Vec2<$T> {}
            unsafe impl bytemuck::Pod for Vec3<$T> {}
            unsafe impl bytemuck::Pod for Vec4<$T> {}    
        )+
    }
}
derive_vectors!(u8, u16, u32, u64, f32, f64);