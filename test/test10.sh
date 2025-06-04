#!/bin/bash
# 测试git merge，参考了⽤例10
# 创建⼀个空⽬录 test10
mkdir test10
# 拷⻉ rust-git 到 test10 ⽬录
cp tests/rust-git test10/
# 进入 test10 ⽬录
cd test10
# 执⾏ rust-git init
./rust-git init
# 创建 main 分⽀并切换到 main 分⽀
./rust-git checkout -b main
# 创建 m.txt
echo "M" > m.txt
# 添加并提交 m.txt
./rust-git add m.txt
hash=$(./rust-git commit -m "main" 2>&1)
# 创建 a 分⽀
./rust-git branch a
# 切换到 a 分⽀
./rust-git checkout a
# 在 a 分⽀中创建 a.txt 文件并添加内容
echo "a" > a.txt
# 添加并提交 a.txt
./rust-git add a.txt
hash1=$(./rust-git commit -m "a" 2>&1)
# 创建 b 分⽀
./rust-git branch b
# 切换到 b 分⽀
./rust-git checkout b
# 在 b 分⽀中创建 b.txt 文件并添加内容
echo "b" > b.txt
# 添加并提交 b.txt
./rust-git add b.txt
hash2=$(./rust-git commit -m "b" 2>&1)
# 切换回 main 分⽀
./rust-git checkout main
# 合并 a 分⽀
content1=$(./rust-git merge a 2>&1)
# 合并 b 分⽀
content2=$(./rust-git merge b 2>&1)
# 验证 main 分⽀是否包含 m.txt、a.txt 和 b.txt 文件
if [ -f "m.txt" ] && [ -f "a.txt" ] && [ -f "b.txt" ]; then
 # 读取文件内容并加入换⾏符
 file_content=$(cat a.txt; echo; cat b.txt)
 echo "$file_content"
else
 echo "Error"
 exit 1
fi