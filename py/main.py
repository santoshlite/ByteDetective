from transformers import AutoProcessor, AutoModelForCausalLM
from PIL import Image
import time
from pathlib import Path
from tzlocal import get_localzone
import datetime
import subprocess
import os
import redis

directory = os.path.expanduser("~")

r = redis.Redis(host='localhost', port=1208, decode_responses=True)

current_timestamp = int(time.time())

picture_extensions = {'.jpg', '.jpeg', '.png', '.tiff', '.gif', '.webp', '.jfif'} # Add more extensions if needed

modified_paths = set()

def get_modified_paths(path, last_updated_formatted):
    home_dir = str(Path.home())  # Get absolute path to the user's home directory

    command = f'find {home_dir} -type d \( -name ".*" -o -name "Library" \) -prune -o -newermt "{last_updated_formatted}" -print 2>/dev/null || true'
    output_bytes = subprocess.check_output(command, shell=True)
    output_str = output_bytes.decode('utf-8')  # Convert bytes to string
    output_list = output_str.split('\n')  # Split the string into a list using newline as delimiter
    for path in output_list:
        file_extension = os.path.splitext(path)[1]
        if path != "" and file_extension.lower() in picture_extensions:
            modified_paths.add(path)

# Get the last_updated timestamp outside the recursive function
last_updated = float(r.get("last_updated")) if r.exists("last_updated") else 947000000 # ~ year 2000 (947000000), just to be safe ;)

local_timezone = get_localzone()
local_datetime = datetime.datetime.fromtimestamp(last_updated, local_timezone)
timestamp_str = local_datetime.strftime("%Y-%m-%d %H:%M:%S")

# Explore the directory and update Redis
get_modified_paths(directory, timestamp_str)

# Create a Redis pipeline
pipe1 = r.pipeline()

# Remove non-existent files/folders from list and collect keys to be deleted
for key in r.keys():
    value = r.get(key)
    if not os.path.exists(value):
        pipe1.delete(key)
    elif value in modified_paths:
        modified_paths.remove(value)
pipe1.execute()

if len(modified_paths) != 0: 
    processor = AutoProcessor.from_pretrained("microsoft/git-base-textcaps")
    model = AutoModelForCausalLM.from_pretrained("microsoft/git-base-textcaps")

    # Create a Redis pipeline
    pipe2 = r.pipeline()

    for picture in modified_paths:
        try:
            image = Image.open(picture)
        except Exception as e:
            continue

        pixel_values = processor(images=image, return_tensors="pt").pixel_values

        generated_ids = model.generate(pixel_values=pixel_values, max_length=50)
        generated_caption = processor.batch_decode(generated_ids, skip_special_tokens=True)[0]
        
        # Queue the set operation in the pipeline
        pipe2.set(generated_caption, picture)
    
    # Execute all queued commands in the pipeline
    pipe2.execute()
    # Update the last_updated timestamp

# Remove the last_updated key from Redis
r.delete("last_updated")
r.set("last_updated", current_timestamp)
print(len(r.keys())-1)
