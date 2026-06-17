from __future__ import annotations

from typing import TYPE_CHECKING

from datafusion import SessionContext,udf

from ._internal import *
from ._internal import ___version

from datafusion_raquet.table_providers import RaquetTable

__version__: str = ___version()

if TYPE_CHECKING:
    from datafusion import SessionContext


class RaquetSessionContext(SessionContext):
    """SessionContext with convenience methods for Raquet tables."""

    def register_raquet(self, name: str, path: str) -> None:
        """Register a Raquet store as a table.

        Args:
            name: Table name to register
            path: Path to the Raquet store (local path or s3:// URL)
        """
        self.register_table(name, RaquetTable(path))

    def register_rastertile(self) -> None:
        """
        """
        from . import rastertile
        self.register_udf(udf(rastertile.TestFromTile()))
        self.register_udf(udf(rastertile.DecodeTile()))
        self.register_udf(udf(rastertile.NativeTile()))
        self.register_udf(udf(rastertile.StatisticsTile()))

    def register_all_quadbin(self) -> None:
        from . import quadbin

        self.register_udf(udf(quadbin.QuadBinFromTile()))
        self.register_udf(udf(quadbin.QuadBinToTile()))
        self.register_udf(udf(quadbin.QuadBinFromLonLat()))
        self.register_udf(udf(quadbin.QuadBinToParent()))
        self.register_udf(udf(quadbin.QuadBinResolution()))
        self.register_udf(udf(quadbin.QuadBinToChildren()))
        self.register_udf(udf(quadbin.QuadBinToSibling()))
        self.register_udf(udf(quadbin.QuadBinKRing()))

        self.register_udf(udf(quadbin.QuadBinToBBOX()))

        self.register_udf(udf(quadbin.QuadBinToBBOXMercator()))
        self.register_udf(udf(quadbin.QuadBinToBBOXWGS84()))
        self.register_udf(udf(quadbin.QuadBinToLonLat()))
        # self.register_udf(udf(quadbin.QuadBinToPixelXY()))
        self.register_udf(udf(quadbin.QuadBinToWKT()))
        self.register_udf(udf(quadbin.QuadBinToGeoJSON()))

  


__all__ = ["RaquetSessionContext", "RaquetTable"]
