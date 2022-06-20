if not exist "build" mkdir build
Xcopy /E/I demo\include build\include
Xcopy /E/I demo\lib build\lib
copy LICENSE build
copy CHANGELOG.md build
