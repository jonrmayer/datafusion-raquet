from raquet_datafusion import RaquetSessionContext

ctx = RaquetSessionContext()

ctx.register_raquet(
    "solar",
    "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet",
)
ctx.register_raquet_table_functions()