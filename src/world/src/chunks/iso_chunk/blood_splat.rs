use byteorder::ReadBytesExt;

#[derive(Debug, Clone, Default)]
pub struct IsoFloorBloodSplat {
    pub xyz: glam::Vec3A,
    // Is i8 in decomp
    pub splat_type: u16,
    pub world_age: f32,
    // Is i8 in decomp
    pub index: u16,
}

impl IsoFloorBloodSplat {
    pub fn read_blood_splat(data: &mut impl std::io::Read) -> Result<Self, std::io::Error> {
        let xyz = glam::Vec3A::new(
            (data.read_u8()? as f32) / 255.0 * 8.0,
            (data.read_u8()? as f32) / 255.0 * 8.0,
            (data.read_u8()? as f32) / 255.0 * 8.0,
        );
        let splat_type = data.read_i8()? as u16;
        let world_age = data.read_f32::<byteorder::BigEndian>()?;
        let index = data.read_i8()? as u16;

        Ok(Self {
            xyz,
            splat_type,
            world_age,
            index,
        })
    }
}
