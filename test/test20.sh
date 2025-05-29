#!/bin/bash
# 创建⼀个空⽬录 test20
mkdir test20
# 拷⻉ git.png 到test20
cp tests/git.png test20
# 拷⻉ rust-git 到 test20 ⽬录
cp tests/rust-git test20/
# 进入 test20 ⽬录
cd test20
# 执⾏ rust-git init
./rust-git init
# 创建 main 分⽀并切换到 main 分⽀
./rust-git checkout -b main
# 添加并提交 git.png
./rust-git add git.png
hash=$(./rust-git commit -m "png" 2>&1)
# 创建 a 分⽀
./rust-git branch a
# 切换到 a 分⽀
./rust-git checkout a
# 创建 a.txt 文件并添加内容
echo "a" > a.txt
# 添加并提交 a.txt
./rust-git add a.txt
hash1=$(./rust-git commit -m "a" 2>&1)
# 切换回 main 分⽀
./rust-git checkout main
# 合并 a 分⽀
content=$(./rust-git merge a 2>&1)
# 删除 a 分⽀
./rust-git branch -d a
# 检查当前⽬录下是否存在 git.png 和 a.txt 文件
if [ ! -f "git.png" ] || [ ! -f "a.txt" ]; then
 echo "Files are missing in the working directory"
 exit 1
fi
# 检查 .git/refs/heads ⽬录下是否存在 main 和 temp 分⽀的引⽤文件
if [ -f ".git/refs/heads/main" ]; then
 echo "Success!"
else
 echo "Branch references are missing in .git/refs/heads"
 exit 1
fi