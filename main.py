from ctypes import cdll
from ctypes import *
from sys import platform

if platform == 'darwin':
    prefix = 'lib'
    ext = 'dylib'
elif platform == 'win32':
    prefix = ''
    ext = 'dll'
else:
    prefix = 'lib'
    ext = 'so'
import qr_searcher

print(qr_searcher.qr_search("empty_images"))


# print(qr_searcher.sum_as_string(1, 2))
# lib = cdll.LoadLibrary('target/release/{}qr_searcher.{}'.format(prefix, ext))
# print(lib.__doc__)
# sum_as_string = lib.sum_as_string_py
# print(sum_as_string(1, 2))




# say_hello = lib.say_hello
# free_query = lib.free_query

# lib.say_hello.restype = c_char_p

# output = say_hello("aaa")
# print(cast(output, c_char_p).value)
# print(type(output))
# free_query(output)