from ctypes import cdll
from ctypes import *
from sys import platform

import qr_searcher

print(qr_searcher.qr_search("test"))