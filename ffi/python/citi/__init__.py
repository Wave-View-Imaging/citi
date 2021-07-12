'''CITI file IO'''

import pkg_resources
from .record import Record


__version__ = pkg_resources.get_distribution("citi").version

__all__ = (
    __version__,
    Record,
)
