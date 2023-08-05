use bevy::{
    math::{DMat3, DMat4, DQuat, DVec3, DVec4},
    prelude::Mesh,
    render::{
        mesh::Indices,
        render_resource::{PrimitiveTopology, VertexFormat},
    },
};

use crate::{Cartesian3, Matrix3, Matrix4, PerspectiveFrustum, PerspectiveOffCenterFrustum};
pub struct FrustumGeometryInfo {
    scratchXDirection: DVec3,
    scratchYDirection: DVec3,
    scratchZDirection: DVec3,
    positions: [f64; 72],
}
#[derive(Clone)]
pub struct FrustumGeometry {
    pub frustum: PerspectiveFrustum,
    pub origin: DVec3,
    pub orientation: DQuat,
}
impl FrustumGeometry {
    pub fn new(frustum: PerspectiveFrustum, origin: DVec3, orientation: DQuat) -> Self {
        Self {
            frustum: frustum,
            origin: origin,
            orientation: orientation,
        }
    }
    pub fn compute_near_planes(&mut self) -> FrustumGeometryInfo {
        let rotationMatrix = DMat3::from_quaternion(&self.orientation);
        let mut x = DVec3::ZERO;
        let mut y = DVec3::ZERO;
        let mut z = DVec3::ZERO;

        x = rotationMatrix.get_column(0).normalize();
        y = rotationMatrix.get_column(1).normalize();
        z = rotationMatrix.get_column(2).normalize();

        x = x.negate();

        let view = DMat4::compute_view(&self.origin, &z, &y, &x);
        let mut positions = [0.0; 72]; //3*4*6
        let projection = self.frustum.get_projection_matrix();
        let viewProjection = *projection * view;
        let inverseViewProjection = viewProjection.inverse();
        let mut frustumSplits = [0.0; 3];
        frustumSplits[0] = self.frustum.near;
        frustumSplits[1] = self.frustum.far;
        let frustumCornersNDC = [
            DVec4::new(-1.0, -1.0, 1.0, 1.0),
            DVec4::new(1.0, -1.0, 1.0, 1.0),
            DVec4::new(1.0, 1.0, 1.0, 1.0),
            DVec4::new(-1.0, 1.0, 1.0, 1.0),
        ];
        for i in 0..2 {
            for j in 0..4 {
                let mut corner = frustumCornersNDC[j].clone();

                corner = inverseViewProjection.multiply_by_vector(&corner);

                // Reverse perspective divide
                let w = 1.0 / corner.w;
                corner = corner * w;
                let mut new_corner = DVec3::new(corner.x, corner.y, corner.z) - self.origin;
                new_corner = new_corner.normalize();

                let fac = z.dot(new_corner);
                new_corner = new_corner * frustumSplits[i] / fac;
                new_corner = new_corner + self.origin;

                positions[12 * i + j * 3] = new_corner.x;
                positions[12 * i + j * 3 + 1] = new_corner.y;
                positions[12 * i + j * 3 + 2] = new_corner.z;
            }
        }
        return FrustumGeometryInfo {
            scratchXDirection: x,
            scratchYDirection: y,
            scratchZDirection: z,
            positions,
        };
    }
    pub fn get_positions(&mut self) -> FrustumGeometryInfo {
        let mut info = self.compute_near_planes();
        let positions = &mut info.positions;
        // -x plane
        let mut offset = 3 * 4 * 2;
        positions[offset] = positions[3 * 4];
        positions[offset + 1] = positions[3 * 4 + 1];
        positions[offset + 2] = positions[3 * 4 + 2];
        positions[offset + 3] = positions[0];
        positions[offset + 4] = positions[1];
        positions[offset + 5] = positions[2];
        positions[offset + 6] = positions[3 * 3];
        positions[offset + 7] = positions[3 * 3 + 1];
        positions[offset + 8] = positions[3 * 3 + 2];
        positions[offset + 9] = positions[3 * 7];
        positions[offset + 10] = positions[3 * 7 + 1];
        positions[offset + 11] = positions[3 * 7 + 2];

        // -y plane
        offset += 3 * 4;
        positions[offset] = positions[3 * 5];
        positions[offset + 1] = positions[3 * 5 + 1];
        positions[offset + 2] = positions[3 * 5 + 2];
        positions[offset + 3] = positions[3];
        positions[offset + 4] = positions[3 + 1];
        positions[offset + 5] = positions[3 + 2];
        positions[offset + 6] = positions[0];
        positions[offset + 7] = positions[1];
        positions[offset + 8] = positions[2];
        positions[offset + 9] = positions[3 * 4];
        positions[offset + 10] = positions[3 * 4 + 1];
        positions[offset + 11] = positions[3 * 4 + 2];

        // +x plane
        offset += 3 * 4;
        positions[offset] = positions[3];
        positions[offset + 1] = positions[3 + 1];
        positions[offset + 2] = positions[3 + 2];
        positions[offset + 3] = positions[3 * 5];
        positions[offset + 4] = positions[3 * 5 + 1];
        positions[offset + 5] = positions[3 * 5 + 2];
        positions[offset + 6] = positions[3 * 6];
        positions[offset + 7] = positions[3 * 6 + 1];
        positions[offset + 8] = positions[3 * 6 + 2];
        positions[offset + 9] = positions[3 * 2];
        positions[offset + 10] = positions[3 * 2 + 1];
        positions[offset + 11] = positions[3 * 2 + 2];

        // +y plane
        offset += 3 * 4;
        positions[offset] = positions[3 * 2];
        positions[offset + 1] = positions[3 * 2 + 1];
        positions[offset + 2] = positions[3 * 2 + 2];
        positions[offset + 3] = positions[3 * 6];
        positions[offset + 4] = positions[3 * 6 + 1];
        positions[offset + 5] = positions[3 * 6 + 2];
        positions[offset + 6] = positions[3 * 7];
        positions[offset + 7] = positions[3 * 7 + 1];
        positions[offset + 8] = positions[3 * 7 + 2];
        positions[offset + 9] = positions[3 * 3];
        positions[offset + 10] = positions[3 * 3 + 1];
        positions[offset + 11] = positions[3 * 3 + 2];
        return info;
    }
}
impl From<FrustumGeometry> for Mesh {
    fn from(value: FrustumGeometry) -> Self {
        let number_of_planes = 6;
        let info = value.clone().get_positions();
        let positions = info.positions;
        let x = info.scratchXDirection;
        let y = info.scratchYDirection;
        let z = info.scratchZDirection;
        let negative_x = x.negate();
        let negative_y = y.negate();
        let negative_z = z.negate();
        let mut endPositions: Vec<[f32; 3]> = Vec::new();
        let mut endNormals: Vec<[f32; 3]> = Vec::new();
        let mut endST: Vec<[f32; 2]> = Vec::new();
        positions.iter().enumerate().step_by(3).for_each(|(i, x)| {
            endPositions.push([
                positions[i] as f32,
                positions[i + 1] as f32,
                positions[i + 2] as f32,
            ])
        });
        let mut indices = (0..number_of_planes * 6).map(|i| 0).collect::<Vec<i32>>();

        for i in 0..number_of_planes {
            let index_offset = i * 6;
            let index = (i * 4) as i32;
            indices[index_offset] = index;
            indices[index_offset + 1] = index + 1;
            indices[index_offset + 2] = index + 2;
            indices[index_offset + 3] = index;
            indices[index_offset + 4] = index + 2;
            indices[index_offset + 5] = index + 3;
        }
        let indices2 = Indices::U32(indices.iter().map(|&x| x as u32).collect());

        let mut normals = [0.0; 72];
        let mut st = [0.0; 48];
        let mut offset = 0;
        getAttributes(offset, &mut normals, &mut st, &negative_z); //near
        offset += 3 * 4;
        getAttributes(offset, &mut normals, &mut st, &z); //far
        offset += 3 * 4;
        getAttributes(offset, &mut normals, &mut st, &negative_x); //-x
        offset += 3 * 4;
        getAttributes(offset, &mut normals, &mut st, &negative_y); //-y
        offset += 3 * 4;
        getAttributes(offset, &mut normals, &mut st, &x); //+x
        offset += 3 * 4;
        getAttributes(offset, &mut normals, &mut st, &y); //+y
        normals.iter().enumerate().step_by(3).for_each(|(i, x)| {
            endNormals.push([
                normals[i] as f32,
                normals[i + 1] as f32,
                normals[i + 2] as f32,
            ]);
        });
        st.iter().enumerate().step_by(2).for_each(|(i, x)| {
            endST.push([st[i] as f32, st[i + 1] as f32]);
        });
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, endPositions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, endNormals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, endST);
        mesh.set_indices(Some(indices2));
        mesh
    }
}
fn getAttributes(offset: usize, normals: &mut [f64; 72], st: &mut [f64; 48], normal: &DVec3) {
    let stOffset = (offset / 3) * 2;
    let mut offset = offset;
    for i in 0..4 {
        normals[offset] = normal.x;
        normals[offset + 1] = normal.y;
        normals[offset + 2] = normal.z;
        offset += 3;
    }

    st[stOffset] = 0.0;
    st[stOffset + 1] = 0.0;
    st[stOffset + 2] = 1.0;
    st[stOffset + 3] = 0.0;
    st[stOffset + 4] = 1.0;
    st[stOffset + 5] = 1.0;
    st[stOffset + 6] = 0.0;
    st[stOffset + 7] = 1.0;
}
#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use crate::{
        math::{equals_epsilon, EPSILON14},
        BoundingSphere, PerspectiveFrustum,
    };

    use super::*;

    #[test]
    fn test() {
        let mut frustum = PerspectiveFrustum::default();
        frustum.fov = 30.0f64.to_radians();
        frustum.aspect_ratio = 1920.0 / 1080.0;
        frustum.near = 1.0;
        frustum.far = 3.0;
        frustum.update_self();
        let mut f = FrustumGeometry::new(frustum, DVec3::ZERO, DQuat::IDENTITY);
        let info = f.get_positions();
        let b = BoundingSphere::from_vertices(info.positions.into());
        assert!(b.radius >= 1.0);
        assert!(b.radius < 2.0);
        assert!(b.center == DVec3::new(0.0, 0.0, 2.0));
    }
}
