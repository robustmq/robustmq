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

# help info
usage() {
    echo "Usage: $0 <module> <action> [config_file]"
    echo "  module: mqtt | journal | place"
    echo "  action: start | stop"
    echo "  config_file: optional, default is config/<module>.toml"
    echo "  example start Placement-Center: $0 place start config/placement-center.toml"
    echo "  example stop Placement-Center: $0 place stop"
    exit 1
}

#get bin_name by mod
get_bin_name() {
    case "${1}" in
        mqtt)    echo "mqtt-server" ;;
        journal) echo "journal-server" ;;
        place)   echo "placement-center" ;;
    esac
}

# check args
[ $# -lt 2 ] && usage


workdir=$(cd $(dirname $0); pwd)
mkdir -p ${workdir}/../logs
mod=$1
action=$2
conf=$3

# check mod
case "${mod}" in
    mqtt|journal|place) ;;
    *) echo "Invalid module type : ${mod}, optional: mqtt, journal, place"; usage ;;
esac

# check action
case "${action}" in
    start|stop) ;;
    *) echo "Invalid action: ${action}, optional: start, stop"; usage ;;
esac

bin_name=$(get_bin_name $mod)
[ -z $conf ] && conf=${workdir}/../config/${bin_name}.toml
log_file=${workdir}/../logs/${bin_name}-nohup.log

start_service() {
  echo "Starting ${bin_name} with config: ${conf}"
  local bin_path="${workdir}/../libs/${bin_name}"
  [ ! -x "$bin_path" ] && echo "ERROR: $bin_path not found or not executable" && exit 1
  [ ! -f $conf ] && echo "Config file not found: $conf" && exit 1
  echo "Config:$conf"
  nohup "${bin_path}" --conf="${conf}" >> "${log_file}" 2>&1 &
  sleep 3
  num=` ps -ef | grep /libs/${bin_name} | grep -v grep | wc -l`
  if [ $num -ge 1 ]
  then
    echo "${bin_name} started successfully."
  else
    echo "WARN: ${bin_name} started failure. please check detail in ${log_file}"
  fi

}

stop_service() {
  no=$(ps -ef | grep "${bin_name}" | grep conf | grep -v grep | awk '{print $2}')
  [ ! -n "$no" ] && echo "No running processes found for ${bin_name}." && exit 0
  echo "Currently running process numbers: $no"
  for pid in $no; do
    echo "Killing process: $pid"
    kill "$pid"
  done
  sleep 3
  num=$(ps -ef | grep "/libs/${bin_name}" | grep conf | grep -v grep | wc -l)
  if [ "$num" -eq 0 ]; then
    echo "${bin_name} stopped successfully."
  else
    echo "WARN: ${bin_name} stop failure."
  fi
}


case "${action}" in
    start) start_service ;;
    stop)  stop_service ;;
esac
