from transformers import AutoProcessor, AutoModelForCausalLM
from PIL import Image
import os
import redis
import time

directory = os.path.expanduser("~")
print(directory)

r = redis.Redis(host='localhost', port=1208, decode_responses=True)

current_timestamp = int(time.time())

# find ~ -type d \( -name ".*" -o -name "Library" \) -prune -o -newermt "2023-06-30 01:30:00" -print    

picture_extensions = {'.jpg', '.jpeg', '.png', '.tiff', '.gif', '.webp', '.jfif'} # Add more extensions if needed

modified_paths = set()

# Function to recursively explore the directory and update the path list
def explore_directory(path):
    for entry in os.scandir(path):
        # Ignore hidden files/folders
        if not entry.name.startswith('.'):
            if entry.is_file():
                file_path = entry.path
                file_extension = os.path.splitext(file_path)[1]
                file_timestamp = os.path.getmtime(file_path)
                if file_extension.lower() in picture_extensions and file_timestamp >= last_updated:
                    # Add to our set
                    modified_paths.add(file_path)
            elif entry.is_dir() and entry.name != "Library":
                # Check the last modified timestamp of the folder
                folder_timestamp = os.path.getmtime(entry.path)
                print(folder_timestamp, entry.path)
                if folder_timestamp >= last_updated: 
                    # Explore the subdirectory recursively
                    explore_directory(entry.path)
        else:
            continue

# Get the last_updated timestamp outside the recursive function
last_updated = float(r.get("last_updated")) if r.exists("last_updated") else 0

print(last_updated)

# Explore the directory and update Redis
explore_directory(directory)

# Remove the last_updated key from Redis
r.delete("last_updated")

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

print(modified_paths)

if len(modified_paths) != 0:
    processor = AutoProcessor.from_pretrained("microsoft/git-base-textcaps")
    model = AutoModelForCausalLM.from_pretrained("microsoft/git-base-textcaps")

    # Create a Redis pipeline
    pipe2 = r.pipeline()

    for picture in modified_paths:
        try:
            image = Image.open(picture)
        except Exception as e:
            print("fail")
            continue

        pixel_values = processor(images=image, return_tensors="pt").pixel_values

        generated_ids = model.generate(pixel_values=pixel_values, max_length=50)
        generated_caption = processor.batch_decode(generated_ids, skip_special_tokens=True)[0]
        
        # Queue the set operation in the pipeline
        pipe2.set(generated_caption, picture)
    
    # Execute all queued commands in the pipeline
    pipe2.execute()

    # Update the last_updated timestamp
r.set("last_updated", current_timestamp)
print(len(r.keys())-1)