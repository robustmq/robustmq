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

if [ "$platform" != "mac" -a "$platform" != "linux" -a "$platform" != "win" -a "$platform" != "arm" ]; then
    echo "platform Error, optional: win,linux, mac, arm"
    exit
fi

if [ -z "$version" ]; then
    echo "package version cannot be null, eg: sh scripts/build-release.sh mac 0.1.0"
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
	cross build --release --target ${arc}

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

	echo "build release package success. ${package_name}.tar.gz "
}

build_linux_release(){
    platform_linux_gnu="x86_64-unknown-linux-gnu"

    platform_linux_musl="x86_64-unknown-linux-musl"
}

build_mac_release(){
    version=$1
    platform_mac="x86_64-apple-darwin"
    package_name="apple-intel-64"
    cross_build $platform_mac $package_name $version
}

build_win_release(){
    platform_win64="x86_64-pc-windows-gnu"


    platform_win32="i686-pc-windows-gnu"
}

build_arm_release(){
    platform_arm64="aarch64-unknown-linux-gnu"
    platform_arm32="armv7-unknown-linux-gnueabihf"
}

if [ "$platform" = "linux" ]; then
   build_linux_release
fi

if [ "$platform" = "mac" ]; then
   build_mac_release $version
fi

if [ "$platform" = "win" ]; then
    build_win_release
fi

if [ "$platform" = "arm" ]; then
    build_arm_release
fi

