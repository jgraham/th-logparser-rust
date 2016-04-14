import ctypes
import sys
import json
from ctypes import c_char_p, Structure, POINTER, c_uint32

prefix = {'win32': ''}.get(sys.platform, 'lib')
extension = {'darwin': '.dylib', 'win32': '.dll'}.get(sys.platform, '.so')
lib = ctypes.cdll.LoadLibrary(prefix + "logparser" + extension)


lib.parse_artifact.argtypes = (c_char_p, c_char_p)
lib.parse_artifact.restype = c_char_p

class ArtifactBuilderCollection(object):
    def __init__(self, url, user_agent="Log Parser"):
        self.url = url
        self.user_agent = user_agent
        self.artifacts = {}
        self.key_map = {
            "job_details": ("Job Info", True)
            "step_data": ("text_log_summary", True)
            "talos_data": ("talos_data", False),
            "performance_data": ("performance_data", False)
        }

    def parse(self):
        data = lib.parse_artifact(self.url, self.user_agent)
        for artifact_str in data.split("\0"):
            artifact = json.parse(artifact_str)
            for key in artifact.keys():
                if key in self.key_map:
                    # Stupid cleanup that should be done on the rust side
                    for subkey in artifact[key].keys():
                        if artifact[key][subkey] is None:
                            del artifact[key][subkey]

                    name, required = self.key_map[key]
                    if not artifact and not required:
                        continue
                    self.artifacts[name] = artifact
