from __future__ import annotations

import duckdb
from datafusion import SessionConfig
from pyarrow import Table
from datafusion_raquet import RaquetSessionContext

config = SessionConfig().with_information_schema(True)

ctx = RaquetSessionContext(config=config)

ctx.register_rastertile()

ctx.register_raquet(
    "solar",
    "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet",
)
# ctx.register_read_raquet()

# sql = "SELECT * from information_schema.tables;"

sql = "SELECT statistics_tile(band_1) from solar where block<>0 ;"

# sql = "SELECT * from my_table_func();"
# # my_table_func
decoded = ctx.sql(sql)
duckdb.sql("SELECT * FROM decoded ").show()
