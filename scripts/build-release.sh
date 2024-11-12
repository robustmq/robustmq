#!/bin/sh
# Copyright 2023 RobustMQ Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

platform=$1
version=$2

if [ "$platform" != "linux-x86" -a "$platform" != "linux-arm" -a "$platform" != "mac" -a "$platform" != "win-x86" -a "$platform" != "win-arm" -a "$platform" != "local" ]; then
    echo "platform Error, optional: linux-x86,linux-arm, mac, win-x86, win-arm"
    exit
fi

if [ -z "$version" ]; then
    echo "package version cannot be null, eg: sh scripts/build-release.sh win-arm 0.1.0"
    exit
fi

cross_build(){
    arc=$1
    platform_package_name=$2
    version=$3

    build="./build"
    target="robustmq"
    
    package_name=${target}-${platform_package_name}-${version}

    echo "${arc} Architecture, start compiling"
    echo "package name: ${package_name}"
    mkdir -p ${build}

    # build
	cargo build --release --target ${arc}

    # makdir fold
	mkdir -p ${build}/${package_name}
	mkdir -p ${build}/${package_name}/bin
	mkdir -p ${build}/${package_name}/libs
	mkdir -p ${build}/${package_name}/config

    # copy bin
	cp -rf target/${arc}/release/mqtt-server ${build}/${package_name}/libs 
	cp -rf target/${arc}/release/placement-center ${build}/${package_name}/libs 
	cp -rf target/${arc}/release/journal-server ${build}/${package_name}/libs 
	cp -rf target/${arc}/release/cli-command-mqtt ${build}/${package_name}/libs 
	cp -rf target/${arc}/release/cli-command-placement ${build}/${package_name}/libs 

    # copy bin&config
	cp -rf bin/* ${build}/${package_name}/bin
	cp -rf config/* ${build}/${package_name}/config

    # chmod file
	chmod -R 777 ${build}/${package_name}/bin/*

    # bundel file
	cd ${build} && tar zcvf ${package_name}.tar.gz ${package_name} && rm -rf ${package_name}
    cd ..
	echo "build release package success. ${package_name}.tar.gz "
}

build_linux_x86_release(){
    version=$1

    # Intel 64
    platform_name="x86_64-unknown-linux-gnu"
    rustup target add ${platform_name}
    package_name="linux-gnu-intel64"
    cross_build $platform_name $package_name $version

    platform_name="x86_64-unknown-linux-musl"
    rustup target add ${platform_name}
    package_name="linux-musl-intel64"
    cross_build $platform_name $package_name $version
}

build_linux_arm_release(){
    version=$1

    platform_name="aarch64-unknown-linux-gnu"
    rustup target add ${platform_name}
    package_name="linux-gnu-arm64"
    cross_build $platform_name $package_name $version

    platform_name="aarch64-unknown-linux-musl"
    rustup target add ${platform_name}
    package_name="linux-musl-arm64"
    cross_build $platform_name $package_name $version
}

build_mac_release(){
    version=$1

    # Intel 64
    platform_name="x86_64-apple-darwin"
    rustup target add ${platform_name}
    package_name="apple-mac-intel64"
    cross_build $platform_name $package_name $version

    # Arm 64
    platform_name="aarch64-apple-darwin"
    rustup target add ${platform_name}
    package_name="apple-mac-arm64"
    cross_build $platform_name $package_name $version
}

build_win_x86_release(){
    version=$1

    # Intel 64
    platform_name="x86_64-pc-windows-gnu"
    rustup target add ${platform_name}
    package_name="windows-gnu-intel64"
    cross_build $platform_name $package_name $version

    # Intel 32
    platform_name="i686-pc-windows-gnu"
    rustup target add ${platform_name}
    package_name="windows-gnu-intel32"
    cross_build $platform_name $package_name $version
}

build_win_arm_release(){
    version=$1
    # Arm 64
    platform_name="aarch64-pc-windows-gnullvm"
    rustup target add ${platform_name}
    package_name="windows-gnu-arm32"
    cross_build $platform_name $package_name $version
}

build_local(){

    build="./build"
    target="robustmq"
    
    package_name=${target}-local

    echo "package name: ${package_name}"
    mkdir -p ${build}

    # build
	cargo build 

    # makdir fold
	mkdir -p ${build}/${package_name}
	mkdir -p ${build}/${package_name}/bin
	mkdir -p ${build}/${package_name}/libs
	mkdir -p ${build}/${package_name}/config

    # copy bin
	cp -rf target/debug/mqtt-server ${build}/${package_name}/libs 
	cp -rf target/debug/placement-center ${build}/${package_name}/libs 
	cp -rf target/debug/journal-server ${build}/${package_name}/libs 
	cp -rf target/debug/cli-command ${build}/${package_name}/libs 

    # copy bin&config
	cp -rf bin/* ${build}/${package_name}/bin
	cp -rf config/* ${build}/${package_name}/config

    # chmod file
	chmod -R 777 ${build}/${package_name}/bin/*

    # bundel file
	cd ${build} && tar zcvf ${package_name}.tar.gz ${package_name} && rm -rf ${package_name}
    cd ..
	echo "build release package success. ${package_name}.tar.gz "
}

if [ "$platform" = "linux-x86" ]; then
   build_linux_x86_release $version
fi

if [ "$platform" = "linux-arm" ]; then
   build_linux_arm_release $version
fi

if [ "$platform" = "mac" ]; then
   build_mac_release $version
fi

if [ "$platform" = "win-x86" ]; then
    build_win_x86_release $version
fi

if [ "$platform" = "win-arm" ]; then
    build_win_arm_release $version
fi

if [ "$platform" = "local" ]; then
    echo "build local"
    rm -rf ../build/
    build_local
fi
