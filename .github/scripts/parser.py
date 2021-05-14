from os import path
from pathlib import Path
from glob import glob
import json
dir_path = path.dirname(path.realpath(__file__))
working_dir = path.abspath("{dir}/../..".format(dir=dir_path))

dd2vtt_glob = "{cwd}/**/*.dd2vtt".format(cwd = working_dir)
dd2vtt_files = glob(dd2vtt_glob, recursive=True)

template = """
# {title}
![image preview](data:image/png;base64,{base64})

## Agreement

All images, assets, and file are provided within accordance what is outline in the repositories license file.
[Please see the license for mre information if you have questions.](https://github.com/dnd-apps/vtt-maps/blob/main/LICENSE)

## Instruction

> [These maps are created with Dungeoundraft, if you are interested please check out that project here.](https://dungeondraft.net/)

*These maps should be able to be imported on FoudryVTT, Roll20, or other applications that support the dd2vtt format.*

### FoundryVTT

1. [Ensure you have the FVTT-DD-Importer installed and enabled.](https://github.com/moo-man/FVTT-DD-Import)
2. Open the scenes section.
3. Go to the scene tab in FVTT, click the Universal Battlemap import button.
4. Fill in the scene name, a path to where the image is to be saved, and the fidelity/offset options.
  a. Fidelity: How many cave walls are used. Far left - less walls, better performance, Far right - more walls, worse performance
  b. Offset: How much to nudge the walls away from the edge.

"""

for dd2vtt_file in dd2vtt_files:
  dd2vtt_file_path = Path(dd2vtt_file)
  dd2vtt_title = dd2vtt_file_path.name.replace(".dd2vtt", "").replace("-", " ").title()
  f = open(dd2vtt_file)
  data = json.loads(f.read())
  f.close()
  base64 = data['image']

  f = open("%s/README.md" % dd2vtt_file_path.parent, "w")
  f.write(template.format(title=dd2vtt_title, base64=base64))
  f.close()
