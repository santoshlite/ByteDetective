'''
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

'''
from tzlocal import get_localzone
import datetime

local_timezone = get_localzone()
local_datetime = datetime.datetime.fromtimestamp(1688545000, local_timezone)
timestamp_str = local_datetime.strftime("%Y-%m-%d %H:%M:%S")
print(timestamp_str)