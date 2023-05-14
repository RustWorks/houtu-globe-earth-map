use crate::{
    getEstimatedLevelZeroGeometricErrorForAHeightmap, getRegularGridAndSkirtIndicesAndEdgeIndices,
    getRegularGridIndicesAndEdgeIndices, CreateVerticeOptions, GeographicTilingScheme,
    HeightmapEncoding, HeightmapTerrainStructure, TerrainMesh, TilingScheme,
};

use super::create_vertice;

pub struct HeightmapTerrainData {
    pub _buffer: Option<Vec<f64>>,
    pub _width: u32,
    pub _height: u32,
    pub _childTileMask: i32,
    pub _encoding: HeightmapEncoding,
    pub _structure: HeightmapTerrainStructure,
    pub _createdByUpsampling: bool,
    pub _waterMask: Option<Vec<u8>>,
    pub _skirtHeight: Option<f64>,
    pub _mesh: Option<TerrainMesh>,
}

impl HeightmapTerrainData {
    pub fn new(
        buffer: Vec<f64>,
        width: u32,
        height: u32,
        childTileMask: Option<i32>,
        encoding: Option<HeightmapEncoding>,
        structure: Option<HeightmapTerrainStructure>,
        createdByUpsampling: Option<bool>,
        waterMask: Option<Vec<u8>>,
        skirtHeight: Option<f64>,
        mesh: Option<TerrainMesh>,
    ) -> Self {
        Self {
            _buffer: Some(buffer),
            _width: width,
            _height: height,
            _childTileMask: childTileMask.unwrap_or(15),
            _encoding: encoding.unwrap_or(HeightmapEncoding::NONE),
            _structure: structure.unwrap_or(HeightmapTerrainStructure::default()),
            _createdByUpsampling: createdByUpsampling.unwrap_or(false),
            _waterMask: waterMask,
            _skirtHeight: skirtHeight,
            _mesh: mesh,
        }
    }
    pub fn _createMeshSync<T>(
        &mut self,
        tilingScheme: GeographicTilingScheme,
        x: u32,
        y: u32,
        level: u32,
        exaggeration: Option<f64>,
        exaggerationRelativeHeight: Option<f64>,
    ) -> &TerrainMesh
    where
        T: TilingScheme,
    {
        let tilingScheme = tilingScheme;
        let x = x;
        let y = y;
        let level = level;
        let exaggeration = exaggeration.unwrap_or(1.0);
        let exaggerationRelativeHeight = exaggerationRelativeHeight.unwrap_or(0.0);

        let ellipsoid = tilingScheme.ellipsoid;
        let nativeRectangle = tilingScheme.tile_x_y_to_native_rectange(x, y, level);
        let rectangle = tilingScheme.tile_x_y_to_rectange(x, y, level);

        // Compute the center of the tile for RTC rendering.
        let center = ellipsoid.cartographicToCartesian(&rectangle.center());

        let structure = self._structure;

        let levelZeroMaxError = getEstimatedLevelZeroGeometricErrorForAHeightmap(
            &ellipsoid,
            self._width,
            tilingScheme.get_number_of_x_tiles_at_level(0),
        );
        let thisLevelMaxError = levelZeroMaxError / (1 << level) as f64;
        let skirtHeight = (thisLevelMaxError * 4.0).min(1000.0);
        self._skirtHeight = Some(skirtHeight);

        let result = create_vertice(CreateVerticeOptions {
            heightmap: self._buffer.unwrap(),
            structure: Some(structure),
            includeWebMercatorT: Some(true),
            width: self._width,
            height: self._height,
            nativeRectangle: nativeRectangle,
            rectangle: Some(rectangle),
            relativeToCenter: Some(center),
            ellipsoid: Some(ellipsoid),
            skirtHeight: skirtHeight,
            isGeographic: Some(true),
            exaggeration: Some(exaggeration),
            exaggerationRelativeHeight: Some(exaggerationRelativeHeight),
        });

        // Free memory received from server after mesh is created.
        self._buffer = None;

        let indicesAndEdges;
        if (skirtHeight > 0.0) {
            indicesAndEdges =
                getRegularGridAndSkirtIndicesAndEdgeIndices(self._width, self._height);
        } else {
            indicesAndEdges = getRegularGridIndicesAndEdgeIndices(self._width, self._height);
        }

        let vertexCountWithoutSkirts = 0;

        // No need to clone here (as we do in the async version) because the result
        // is not coming from a web worker.
        self._mesh = Some(TerrainMesh::new(
            center,
            result.vertices,
            indicesAndEdges.indices,
            indicesAndEdges.indexCountWithoutSkirts,
            vertexCountWithoutSkirts,
            result.minimumHeight,
            result.maximumHeight,
            result.boundingSphere3D,
            result.occludeePointInScaledSpace,
            result.encoding.stride,
            result.orientedBoundingBox,
            result.encoding,
            indicesAndEdges.westIndicesSouthToNorth,
            indicesAndEdges.southIndicesEastToWest,
            indicesAndEdges.eastIndicesNorthToSouth,
            indicesAndEdges.northIndicesWestToEast,
        ));

        return &self._mesh.unwrap();
    }
}
