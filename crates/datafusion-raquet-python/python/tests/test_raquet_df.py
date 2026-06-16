from __future__ import annotations

import duckdb
from datafusion import SessionConfig
from pyarrow import Table
from datafusion_raquet import RaquetSessionContext

config = SessionConfig().with_information_schema(True)

ctx = RaquetSessionContext(config=config)

ctx.register_rastertile()
ctx.register_all_quadbin()

ctx.register_raquet(
    "solar",
    "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet",
)
# ctx.register_read_raquet()

# sql = "SELECT * from information_schema.tables;"

# sql = """imi

# create table tile_stats as 
# select block,quadbin_resolution(cast(block as BIGINT)) resolution,statistics_tile(band_1) as stats from solar where block<>0 ;




# """
# tilestats=ctx.sql(sql)
# ctx.register_table('tile_stats',tilestats)

# sql = "SELECT unnest(statistics_tile(band_1)) as stats from solar where block<>0 ;"

sql = """
with data as (
select native_tile(band_1) native from solar where block<>0
)
select array_element(native,10) from data

"""

# sql = "select native_tile(band_1) native from solar where block<>0 limit 1"
# sql = "SELECT * from information_schema.tables;"

import time
start = time.time()
decoded = ctx.sql(sql)
# decoded.show()
end = time.time()

duckdb.sql("SELECT * FROM decoded").show()
elapsed = end - start

print(f'Dataframe Time taken: {elapsed:.6f} seconds')
