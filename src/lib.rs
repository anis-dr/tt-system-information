#![deny(clippy::all)]

extern crate humansize;
#[macro_use]
extern crate napi_derive;

use battery::Manager;
use humansize::file_size_opts::{BINARY, DECIMAL};
use humansize::FileSize;
use nvml_wrapper::enum_wrappers::device::{Brand, TemperatureSensor};
use nvml_wrapper::NVML;
use sysinfo::{ComponentExt, CpuExt, DiskExt, DiskType, NetworkExt, NetworksExt, Pid, PidExt, ProcessExt, System, SystemExt};

#[napi(object)]
pub struct ThreadInfo {
    pub name: String,
    pub cpu_usage: f64,
    pub frequency: u32,
    pub vendor_id: String,
    pub brand: String,
}

#[napi(object)]
pub struct CpuInfo {
    pub name: String,
    pub cpu_usage: f64,
    pub frequency: u32,
    pub vendor_id: String,
    pub brand: String,
    pub physical_core_count: u32,
    pub threads: Vec<ThreadInfo>,
}

#[napi]
fn cpu_info() -> CpuInfo {
    let mut sys = System::new_all();
    sys.refresh_cpu();

    let global_cpu = sys.global_cpu_info();

    CpuInfo {
        name: global_cpu.name().to_string(),
        cpu_usage: global_cpu.cpu_usage() as f64,
        frequency: global_cpu.frequency() as u32,
        vendor_id: global_cpu.vendor_id().to_string(),
        brand: global_cpu.brand().to_string(),
        physical_core_count: sys.physical_core_count().unwrap_or(0) as u32,
        threads: sys
            .cpus()
            .iter()
            .map(|cpu| ThreadInfo {
                name: cpu.name().to_string(),
                cpu_usage: cpu.cpu_usage() as f64,
                frequency: cpu.frequency() as u32,
                vendor_id: cpu.vendor_id().to_string(),
                brand: cpu.brand().to_string(),
            })
            .collect(),
    }
}

#[napi(object)]
pub struct DiskInfo {
    pub name: String,
    pub total_space: String,
    pub available_space: String,
    pub is_removable: bool,
    pub type_: String,
}


#[napi]
fn disks_info() -> Vec<DiskInfo> {
    let mut sys = System::new_all();
    sys.refresh_disks();

    sys.disks().iter().map(|disk| {
        let type_ = disk.type_();
        let type_str = match type_ {
            DiskType::HDD => "HDD",
            DiskType::SSD => "SSD",
            DiskType::Unknown(_) => "Unknown",
        };

        DiskInfo {
            name: disk.name().to_str().unwrap_or("").to_string(),
            total_space: disk.total_space()
                .file_size(DECIMAL)
                .unwrap_or_else(|_| String::from("")),
            available_space: disk.available_space()
                .file_size(DECIMAL)
                .unwrap_or_else(|_| String::from("")),
            is_removable: disk.is_removable(),
            type_: type_str.to_string(),
        }
    }).collect()
}

#[napi(object)]
struct NetworkInfo {
    pub name: String,
    pub received: i64,
    pub total_received: i64,
    pub transmitted: i64,
    pub total_transmitted: i64,
    pub packets_received: i64,
    pub packets_transmitted: i64,
    pub total_packets_transmitted: i64,
    pub errors_on_received: i64,
    pub total_errors_on_received: i64,
    pub errors_on_transmitted: i64,
    pub total_errors_on_transmitted: i64,
}

#[napi]
fn networks_info() -> Vec<NetworkInfo> {
    let mut sys = System::new_all();
    sys.refresh_networks();

    sys.networks().iter().map(|(id, network)| {
        NetworkInfo {
            name: id.to_string(),
            received: network.received() as i64,
            total_received: network.total_received() as i64,
            transmitted: network.transmitted() as i64,
            total_transmitted: network.total_transmitted() as i64,
            packets_received: network.packets_received() as i64,
            packets_transmitted: network.packets_transmitted() as i64,
            total_packets_transmitted: network.total_packets_transmitted() as i64,
            errors_on_received: network.errors_on_received() as i64,
            total_errors_on_received: network.total_errors_on_received() as i64,
            errors_on_transmitted: network.errors_on_transmitted() as i64,
            total_errors_on_transmitted: network.total_errors_on_transmitted() as i64,
        }
    }).collect()
}

#[napi(object)]
pub struct LoadAverageInfo {
    #[napi(js_name = "1")]
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[napi]
fn load_average_info() -> LoadAverageInfo {
    let sys = System::new_all();
    let load_average = sys.load_average();
    LoadAverageInfo {
        one: load_average.one,
        five: load_average.five,
        fifteen: load_average.fifteen,
    }
}

#[napi(object)]
pub struct MemoryInfo {
    pub total_memory: String,
    pub available_memory: String,
    pub used_memory: String,
    pub free_memory: String,
    pub total_swap: String,
    pub free_swap: String,
    pub used_swap: String,
}

fn get_memory_value(value: u64) -> String {
    ((value as f64 * 0.9765625) as u64 * 1024)
        .file_size(BINARY)
        .unwrap_or_else(|_| "".to_string())
}

#[napi]
fn memory_info() -> MemoryInfo {
    let mut sys = System::new_all();
    sys.refresh_memory();

    MemoryInfo {
        total_memory: get_memory_value(sys.total_memory()),
        available_memory: get_memory_value(sys.available_memory()),
        used_memory: get_memory_value(sys.used_memory()),
        free_memory: get_memory_value(sys.free_memory()),
        total_swap: get_memory_value(sys.total_swap()),
        free_swap: get_memory_value(sys.free_swap()),
        used_swap: get_memory_value(sys.used_swap()),
    }
}

#[napi(object)]
pub struct OsInfo {
    pub name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub host_name: String,
}

#[napi]
fn os_info() -> OsInfo {
    let sys = System::new_all();
    OsInfo {
        name: sys.name()
            .unwrap_or_else(|| "Unknown".to_string()),
        kernel_version: sys.kernel_version()
            .unwrap_or_else(|| "Unknown".to_string()),
        os_version: sys.os_version()
            .unwrap_or_else(|| "Unknown".to_string()),
        host_name: sys.host_name()
            .unwrap_or_else(|| "Unknown".to_string()),
    }
}

#[napi(object)]
pub struct ProcessInfo {
    pub name: String,
    pub cmd: String,
    pub path: String,
    pub pid: u32,
    pub ppid: u32,
    pub env_var: Vec<String>,
    pub cwd: String,
    pub root: String,
    pub memory: i64,
    pub virtual_memory: i64,
    pub running_time: i64,
    pub status: String,
    pub disk_usage: DiskUsage,
    pub cpu_usage: i64,
}

#[napi(object)]
pub struct DiskUsage {
    pub read_bytes: i64,
    pub written_bytes: i64,
    pub total_read_bytes: i64,
    pub total_written_bytes: i64,
}

#[napi]
fn processes_info() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    sys.processes().iter().map(|(_id, process)| {
        let disk_usage = process.disk_usage();
        let disk_usage_result = DiskUsage {
            read_bytes: disk_usage.read_bytes as i64,
            written_bytes: disk_usage.written_bytes as i64,
            total_read_bytes: disk_usage.total_read_bytes as i64,
            total_written_bytes: disk_usage.total_written_bytes as i64,
        };

        ProcessInfo {
            name: process.name().to_string(),
            cmd: process.cmd().join(" "),
            path: process.exe().to_str().unwrap_or("").to_string(),
            pid: process.pid().as_u32(),
            ppid: process.parent().unwrap_or_else(|| Pid::from(-1)).as_u32(),
            env_var: Vec::from(process.environ()),
            cwd: process.cwd().to_str().unwrap_or("").to_string(),
            root: process.root().to_str().unwrap_or("").to_string(),
            memory: process.memory() as i64,
            virtual_memory: process.virtual_memory() as i64,
            running_time: process.run_time() as i64,
            status: process.status().to_string(),
            disk_usage: disk_usage_result,
            cpu_usage: process.cpu_usage() as i64,
        }
    }).collect()
}

#[napi(object)]
pub struct BatteryInfo {
    pub vendor: String,
    pub model: String,
    pub state: String,
    pub temperature: String,
    pub cycle_count: u32,
    pub energy_full_design: String,
    pub energy_full: String,
    pub energy: String,
    pub energy_rate: String,
    pub voltage: String,
    pub state_of_charge: String,
    pub state_of_health: String,
}

#[napi]
fn battery_info() -> Vec<BatteryInfo> {
    let manager = Manager::new().unwrap();
    let batteries = manager.batteries().unwrap();

    let mut batteries_res = Vec::new();
    for (_i, battery) in batteries.enumerate() {
        let battery_info = battery.unwrap();

        let temperature = battery_info.temperature().unwrap_or_default();
        let temp_celsius = ((temperature.value - 273.15) * 100.0).trunc() / 100.0;

        batteries_res.push(BatteryInfo {
            vendor: battery_info.vendor().unwrap_or("Unknown").to_string(),
            model: battery_info.model().unwrap_or("Unknown").to_string(),
            state: battery_info.state().to_string(),
            temperature: temp_celsius.to_string() + "°C",
            cycle_count: battery_info.cycle_count().unwrap_or_default(),
            energy_full_design: battery_info.energy_full_design().value.to_string() + " Joule",
            energy_full: battery_info.energy_full().value.to_string() + " Joule",
            energy: battery_info.energy().value.to_string() + " Joule",
            energy_rate: battery_info.energy_rate().value.to_string() + " Watt",
            voltage: battery_info.voltage().value.to_string() + " Volt",
            state_of_charge: (battery_info.state_of_charge().value * 100.0).to_string() + "%",
            state_of_health: (battery_info.state_of_health().value * 100.0).to_string() + "%",
        });
    };
    batteries_res
}

#[napi(object)]
struct GraphicsInfo {
    pub brand: String,
    pub name: String,
    pub fan_speed: String,
    pub power_limit: String,
    pub memory: GpuMemory,
    pub temperature: String,
    pub utilization: GpuUsage,
}

#[napi(object)]
struct GpuMemory {
    pub total: String,
    pub free: String,
    pub used: String,
}

#[napi(object)]
struct GpuUsage {
    pub gpu: String,
    pub memory: String,
}

#[napi]
fn graphics_info() -> Vec<GraphicsInfo> {
    let nvml_result = NVML::init();
    let nvml = nvml_result;
    let nvml = match nvml {
        Ok(nvml) => nvml,
        Err(_) => {
            return Vec::new();
        }
    };

    let count = nvml.device_count().unwrap();
    let mut graphics_info = Vec::new();
    for i in 0..count {
        let device = nvml.device_by_index(i).unwrap();

        let brand = device.brand().unwrap();
        let brand = match brand {
            Brand::Unknown => "Unknown",
            Brand::Quadro => "Quadro",
            Brand::Tesla => "Tesla",
            Brand::NVS => "NVS",
            Brand::GRID => "GRID",
            Brand::GeForce => "GeForce",
            Brand::Titan => "Titan",
        };

        let name = device.name().unwrap();
        let fan_speed = device.fan_speed(0).unwrap();
        let power_limit = device.enforced_power_limit().unwrap();
        let memory = device.memory_info().unwrap();
        let memory = GpuMemory {
            total: memory.total.to_string() + " Bytes",
            free: memory.free.to_string() + " Bytes",
            used: ((memory.total as f64 / memory.free as f64) * 100.0).trunc().to_string() + "%",
        };
        let temperature = device.temperature(TemperatureSensor::Gpu).unwrap();
        let utilization = device.utilization_rates().unwrap();
        let utilization = GpuUsage {
            gpu: utilization.gpu.to_string() + "%",
            memory: utilization.memory.to_string() + "%",
        };

        graphics_info.push(GraphicsInfo {
            brand: brand.to_string(),
            name: name.to_string(),
            fan_speed: fan_speed.to_string() + " RPM",
            power_limit: power_limit.to_string() + " Watts",
            memory,
            temperature: temperature.to_string() + "°C",
            utilization,
        });
    }
    graphics_info
}

#[napi(object)]
pub struct ComponentInfo {
    pub label: String,
    pub temperature: String,
}

#[napi]
fn components_info() -> Vec<ComponentInfo> {
    let mut sys = System::new_all();
    sys.refresh_components();

    let components = sys.components();

    let mut components_info = Vec::new();
    for component in components {
        components_info.push(ComponentInfo {
            label: component.label().to_string(),
            temperature: component.temperature().to_string() + "°C",
        });
    }
    components_info
}