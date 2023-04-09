# To compile and test citi

1. Git clone citi from github
2. Check out to Raspberry branch by:
    ```
    git checkout Raspi-dev
    ```
3. Create build folder and cd to the folder by:
    ```
    mkdir build
    cd build
    ```
4. Run cmake in the build folder by:
    ```
    cmake ..
    ```
5. Run make in the build folder by:
    ```
    make
    ```
6. Go back to root and test the citi by run test_exec:
    ```
    cd ..
    ./build/ffi/cpp/tests/test_exec 
    ```
7. Test rust from root by run:
    ```
    cargo test
    ```


