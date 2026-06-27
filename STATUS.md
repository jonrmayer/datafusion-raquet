### QUADBIN Functions

| Function | Description | Return |
|----------|-------------|--------|
| `quadbin_from_tile(x, y, z)` | Tile coordinates to QUADBIN | `UBIGINT` |
| `quadbin_to_tile(cell)` | QUADBIN to tile coordinates | `STRUCT(x, y, z)` |
| `quadbin_from_lonlat(lon, lat, resolution)` | Lon/lat to QUADBIN cell | `UBIGINT` |
| `quadbin_to_lonlat(cell)` | Cell center as lon/lat | `STRUCT(lon, lat)` |
| `quadbin_resolution(cell)` | Get resolution level | `INTEGER` |
| `quadbin_to_bbox(cell)` | Bounding box of cell | `STRUCT(...)` |
| `quadbin_pixel_xy(lon, lat, res, tile_size)` | Pixel coordinates within tile | `STRUCT(pixel_x, pixel_y)` |
| `quadbin_to_parent(cell)` | Parent cell (resolution - 1) | `UBIGINT` |
| `quadbin_to_parent(cell, resolution)` | Parent at specific resolution | `UBIGINT` |
| `quadbin_to_children(cell)` | 4 children at resolution + 1 | `LIST(UBIGINT)` |
| `quadbin_to_children(cell, resolution)` | Children at specific resolution | `LIST(UBIGINT)` |
| `quadbin_sibling(cell)` | Sibling cells (same parent) | `LIST(UBIGINT)` |
| `quadbin_kring(cell, k)` | Cells within k distance | `LIST(UBIGINT)` |
| `quadbin_polyfill(wkt, resolution)` | Fill geometry from wkt with cells | `LIST(UBIGINT)` |
| `quadbin_to_wkt(cell)` | Cell as WKT POLYGON | `VARCHAR` |
| `quadbin_to_geojson(cell)` | Cell as GeoJSON | `VARCHAR` |


### Raster TILE Functions

| Function | Description | Return |
|----------|-------------|--------|
| `decompress_tile(band)` | Decompress band bytes | `[]byte` |
| `decode_tile(band)` | Decode band bytes to []Float64 | `[]Float64` |
| `native_tile(band)` | Decode band bytes to native data type eg []Int16 | `[]Native` |
| `statistics_tile(band)` | Summary statistics | `STRUCT(valid_count, sum, mean, min, max, stddev)` |
| `raquet_pixel(band,pixel_x,pixel_y)` | Pixel by x,y | `Float64` |
| `raquet_value(block,band,wkt)` | Pixel Value by WKT | `Float64` |

### read_raquet* Functions (File I/O)

| Function | Description |
|----------|-------------|
| `read_raquet(table_name)` | Read all data rows |
| `read_raquet_metadata(table_name)` | Read metadata row only |
| `read_raquet_at(table_name,wkt)` | Fetch tiles intersecting with POINT/POLYGON WKT |