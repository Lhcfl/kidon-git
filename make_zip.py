import zipfile
import os

EXCLUDE_DIRS = { ".kidon-git", ".git", "target", "test", "rust-git.zip" }

def should_exclude(path: str):
    parts = path.split(os.sep)
    return any(part in EXCLUDE_DIRS for part in parts)

def zipdir(path, ziph, arc_path_prefix=""):
    for root, dirs, files in os.walk(path):
        # 过滤掉不想进入的子目录（就地修改 dirs）
        dirs[:] = [d for d in dirs if not should_exclude(os.path.relpath(os.path.join(root, d), path))]
        for file in files:
            rel_dir = os.path.relpath(root, path)
            rel_file = os.path.join(rel_dir, file)
            if should_exclude(rel_file):
                continue
            abs_file_path = os.path.join(root, file)
            arcname = os.path.join(arc_path_prefix, rel_file.removeprefix(".\\"))
            print(f"zip: {abs_file_path} -> {arcname}")
            ziph.write(abs_file_path, arcname)

folder = "."
arc_path_prefix="rust-git"

try:
  os.remove("rust-git.zip")  # 删除旧的 zip 文件
except FileNotFoundError:
  pass

with zipfile.ZipFile("rust-git.zip", 'w', zipfile.ZIP_DEFLATED) as ziph:
    for root, dirs, files in os.walk(folder):
      # 过滤掉不想进入的子目录（就地修改 dirs）
      dirs[:] = [d for d in dirs if not should_exclude(os.path.relpath(os.path.join(root, d), folder))]
      for file in files:
          rel_dir = os.path.relpath(root, folder)
          rel_file = os.path.join(rel_dir, file)
          if should_exclude(rel_file):
              continue
          abs_file_path = os.path.join(root, file)
          arcname = os.path.join(arc_path_prefix, rel_file.removeprefix(".\\"))
          print(f"zip: {abs_file_path} -> {arcname}")
          ziph.write(abs_file_path, arcname)
    
    try:
      ziph.write("target/release/rust-git", "target/release/rust-git")
      ziph.write("target/debug/rust-git", "target/release/rust-git")
    except FileNotFoundError:
      print("No binary found in target/release, you should build it first.")
      exit(1)