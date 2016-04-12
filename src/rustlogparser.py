import ctypes
import sys
import json
from ctypes import c_char_p, Structure, POINTER, c_uint32

prefix = {'win32': ''}.get(sys.platform, 'lib')
extension = {'darwin': '.dylib', 'win32': '.dll'}.get(sys.platform, '.so')
lib = ctypes.cdll.LoadLibrary(prefix + "logparser" + extension)


class StepParserStruct(Structure):
    pass

lib.step_parser_new.restype = POINTER(StepParserStruct)
lib.step_parser_free.argtypes = (POINTER(StepParserStruct),)
lib.step_parser_clear.argtypes = (POINTER(StepParserStruct),)
lib.step_parser_parse_line.args = (POINTER(StepParserStruct), c_char_p, c_uint32)
lib.step_parser_finish_parse.args = (POINTER(StepParserStruct), c_uint32)
lib.step_parser_get_artifact.argtypes = (POINTER(StepParserStruct),)
lib.step_parser_get_artifact.restype = c_char_p
lib.step_parser_free_artifact.argtypes = (c_char_p,)

class StepParser(object):
    def __init__(self, name):
        self.name = name
        self.obj = None

    def __enter__(self):
        self.obj = lib.step_parser_new()

    def __exit__(self, *args, **kwargs):
        if self.obj:
            lib.step_parser_free(self.obj)
            self.obj = None

    def clear(self):
        lib.step_parser_clear(self.obj)
        self.complete = False

    def parse_line(self, line, line_number):
        lib.step_parser_parse_line(self.obj, line, line_number)

    def finish_parse(self, last_line_number):
        lib.step_parser_finish_parse(self.obj, last_line_number)

    def get_artifact(self):
        artifact = lib.step_parser.get_artifact(self.obj)
        rv = json.loads(artifact)
        lib.step_parser_free_artifact(artifact)
        return rv
