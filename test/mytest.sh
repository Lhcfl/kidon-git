mkdir mytest
cp tests/rust-git mytest/
cd mytest

./rust-git init
echo "This is a test file" > test.txt
./rust-git add test.txt
./rust-git commit -m "Add test file"

./rust-git checkout -b test-branch1
echo "This is a test file in branch 1" > test.txt
./rust-git add test.txt
./rust-git commit -m "Update test file in branch 1"

./rust-git checkout master
./rust-git checkout -b test-branch2

echo "This is a test file in branch 2" > test.txt
./rust-git add test.txt
./rust-git commit -m "Update test file in branch 2"

./rust-git merge test-branch1