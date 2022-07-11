if not exist "install" mkdir install
Xcopy /E/I demo\sdk install\sdk
copy LICENSE install\sdk
copy CHANGELOG.md install\sdk
