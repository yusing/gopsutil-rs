mod gotypes;

pub use psutil::*;
use std::{ffi::OsStr, os::raw::c_float, sync::LazyLock};

use crate::gotypes::GoString;

static mut CPU_PERCENT_COLLECTOR: LazyLock<psutil::cpu::CpuPercentCollector> =
    std::sync::LazyLock::new(|| psutil::cpu::CpuPercentCollector::new().unwrap());
static mut DISK_IO_COUNTERS_COLLECTOR: LazyLock<psutil::disk::DiskIoCountersCollector> =
    std::sync::LazyLock::new(psutil::disk::DiskIoCountersCollector::default);

#[unsafe(no_mangle)]
pub extern "C" fn gopsutil_cpu_percent(out: &mut c_float) -> bool {
    unsafe {
        // it will not be used concurrently
        #[allow(static_mut_refs)]
        match CPU_PERCENT_COLLECTOR.cpu_percent() {
            Ok(usage) => {
                *out = usage;
                true
            }
            Err(_) => false,
        }
    }
}

#[repr(C)]
pub struct Memory {
    total: u64,
    available: u64,
    used: u64,
    used_percent: f32,
    free: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn gopsutil_memory(out: &mut Memory) -> bool {
    match psutil::memory::virtual_memory() {
        Ok(mem) => {
            out.total = mem.total();
            out.available = mem.available();
            out.used = mem.used();
            out.used_percent = mem.percent();
            out.free = mem.free();
            true
        }
        Err(_) => false,
    }
}

#[repr(C)]
pub struct DiskUsageStat {
    device: GoString,
    path: GoString,
    fstype: GoString,
    total: u64,
    free: u64,
    used: u64,
    used_percent: f32,
}

#[unsafe(no_mangle)]
pub extern "C" fn gopsutil_disk_usage(path: &GoString, out: &mut DiskUsageStat) -> bool {
    let path: &OsStr = path.into();
    match psutil::disk::disk_usage(path) {
        Ok(usage) => {
            // device and fstype are not available here
            out.path = path.into();
            out.total = usage.total();
            out.free = usage.free();
            out.used = usage.used();
            out.used_percent = usage.percent();
            true
        }
        Err(_) => false,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn gopsutil_disk_usage_by_partition(
    m: &gotypes::GoMapHeader,
    m_type: &gotypes::GoMapType,
) -> bool {
    assert_eq!(m_type.elem.size, std::mem::size_of::<DiskUsageStat>());
    match psutil::disk::partitions_physical() {
        Ok(partitions) => {
            for partition in &partitions {
                if let Ok(usage) = psutil::disk::disk_usage(partition.mountpoint()) {
                    let stats = &DiskUsageStat {
                        device: partition.device().into(),
                        path: partition.mountpoint().into(),
                        fstype: partition.filesystem().as_str().into(),
                        total: usage.total(),
                        free: usage.free(),
                        used: usage.used(),
                        used_percent: usage.percent(),
                    };
                    gotypes::go_strmap_set(
                        m,
                        m_type,
                        partition.mountpoint().into(),
                        gotypes::any_to_go_ptr(stats),
                    )
                }
            }
            true
        }
        Err(_) => false,
    }
}

#[repr(C)]
pub struct DiskIOCountersStat {
    name: GoString,
    read_bytes: u64,
    write_bytes: u64,
    read_count: u64,
    write_count: u64,
    // below will be calculated in the caller
    iops: u64,
    read_speed: f32,
    write_speed: f32,
}

// struct DiskIOState {
//     last_counters: HashMap<String, DiskIoCounters>,
//     last_time: Instant,
// }

// impl DiskIOState {
//     fn new() -> Self {
//         Self {
//             last_counters: HashMap::new(),
//             last_time: Instant::now(),
//         }
//     }

//     fn get_last_counters(&self) -> &HashMap<String, DiskIoCounters> {
//         &self.last_counters
//     }

//     fn get_last_time(&self) -> Instant {
//         self.last_time
//     }

//     fn set_last_counters(&mut self, counters: HashMap<String, DiskIoCounters>) {
//         self.last_counters = counters;
//     }

//     fn set_last_time(&mut self, time: Instant) {
//         self.last_time = time;
//     }

//     fn calc_io(&self, io: &mut DiskIOCountersStat, ts: Instant) {
//         let last_time = self.get_last_time();
//         let elapsed = ts.duration_since(last_time);
//         let name: &str = (&io.name).into();
//         if let Some(counters) = self.get_last_counters().get(name) {
//             io.read_speed =
//                 ((io.read_bytes - counters.read_bytes()) as f32) / elapsed.as_secs_f32();
//             io.write_speed =
//                 ((io.write_bytes - counters.write_bytes()) as f32) / elapsed.as_secs_f32();
//             // just in case, use abs_diff instead of -
//             let rps = io.read_count.abs_diff(counters.read_count()) / elapsed.as_secs();
//             let wps = io.write_count.abs_diff(counters.write_count()) / elapsed.as_secs();
//             io.iops = rps + wps;
//         }
//     }
// }

// static mut DISK_IO_STATE: LazyLock<DiskIOState> = std::sync::LazyLock::new(DiskIOState::new);

fn should_exclude_disk(name: &str) -> bool {
    // include only sd* and nvme* disk devices / partitions

    if name.len() < 3 {
        return true;
    }

    if name.starts_with("nvme") || name.starts_with("mmcblk") {
        // NVMe/SD/MMC
        return false;
    }

    match name.chars().next().unwrap_or('\0') {
        's' | 'h' | 'v' => {
            // SCSI/SATA/virtio disks
            if name.chars().nth(1).unwrap_or('\0') != 'd' {
                return true;
            }
        }
        'x' => {
            // Xen virtual disks
            if !name.starts_with("xvd") {
                return true;
            }
        }
        _ => return true,
    }
    false
}

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
pub extern "C" fn gopsutil_disk_io_counters_by_partition(
    m: &gotypes::GoMapHeader,
    m_type: &gotypes::GoMapType,
) -> bool {
    assert_eq!(m_type.elem.size, std::mem::size_of::<DiskIOCountersStat>());
    match unsafe { DISK_IO_COUNTERS_COLLECTOR.disk_io_counters_per_partition() } {
        Ok(partitions) => {
            // let now = Instant::now();
            for (partition, io) in &partitions {
                if should_exclude_disk(partition) {
                    continue;
                }
                let counters = &mut DiskIOCountersStat {
                    name: partition.into(),
                    read_bytes: io.read_bytes(),
                    write_bytes: io.write_bytes(),
                    read_count: io.read_count(),
                    write_count: io.write_count(),
                    iops: 0,
                    read_speed: 0.0,
                    write_speed: 0.0,
                };
                // unsafe { DISK_IO_STATE.calc_io(counters, now) };
                gotypes::go_strmap_set(
                    m,
                    m_type,
                    partition.into(),
                    gotypes::any_to_go_ptr(counters),
                );
            }
            // unsafe {
            //     DISK_IO_STATE.set_last_counters(partitions);
            //     DISK_IO_STATE.set_last_time(now);
            // }
            true
        }
        Err(_) => false,
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NetIOCountersStat {
    bytes_sent: u64,
    bytes_recv: u64,
    upload_speed: f32,
    download_speed: f32,
}

// struct NetIOState {
//     last_counter: NetIOCountersStat,
//     last_time: Instant,
// }

// impl NetIOState {
//     fn new() -> Self {
//         Self {
//             last_counter: NetIOCountersStat {
//                 bytes_sent: 0,
//                 bytes_recv: 0,
//                 upload_speed: 0.0,
//                 download_speed: 0.0,
//             },
//             last_time: Instant::now(),
//         }
//     }

//     fn get_last_counter(&self) -> &NetIOCountersStat {
//         &self.last_counter
//     }

//     fn get_last_time(&self) -> Instant {
//         self.last_time
//     }

//     fn set_last_counter(&mut self, counter: NetIOCountersStat) {
//         self.last_counter = counter;
//     }

//     fn set_last_time(&mut self, time: Instant) {
//         self.last_time = time;
//     }

//     fn calc_io(&self, counters: &mut NetIOCountersStat, ts: Instant) {
//         let last_time = self.get_last_time();
//         let elapsed = ts.duration_since(last_time);
//         let last = self.get_last_counter();
//         counters.upload_speed =
//             ((counters.bytes_sent - last.bytes_sent) as f32) / elapsed.as_secs_f32();
//         counters.download_speed =
//             ((counters.bytes_recv - last.bytes_recv) as f32) / elapsed.as_secs_f32();
//     }
// }

// static mut NET_IO_STATE: LazyLock<NetIOState> = std::sync::LazyLock::new(NetIOState::new);
static mut NET_IO_COUNTERS_COLLECTOR: LazyLock<psutil::network::NetIoCountersCollector> =
    std::sync::LazyLock::new(psutil::network::NetIoCountersCollector::default);

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
pub extern "C" fn gopsutil_net_io_counters(out: &mut NetIOCountersStat) -> bool {
    match unsafe { NET_IO_COUNTERS_COLLECTOR.net_io_counters() } {
        Ok(counters) => {
            // let now = Instant::now();
            out.bytes_sent = counters.bytes_sent();
            out.bytes_recv = counters.bytes_recv();
            // unsafe {
            //     NET_IO_STATE.calc_io(out, now);
            //     NET_IO_STATE.set_last_counter(*out);
            //     NET_IO_STATE.set_last_time(now);
            // }
            true
        }
        Err(_) => false,
    }
}

#[repr(C)]
pub struct TemperatureStat {
    sensor_key: GoString,
    temperature: f32,
    high: f32,
    critical: f32,
}

pub fn format_sensor_key(sensor: &psutil::sensors::TemperatureSensor) -> String {
    if let Some(label) = sensor.label() {
        format!(
            "{}_{}",
            sensor.unit(),
            label.replace(" ", "_").to_lowercase()
        )
    } else {
        sensor.unit().to_string()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn gopsutil_temperatures(
    out: &gotypes::GoSlice,
    elem_type: &gotypes::GoType,
) -> bool {
    assert_eq!(elem_type.size, std::mem::size_of::<TemperatureStat>());
    let flattened = psutil::sensors::temperatures().into_iter().flatten();
    let stats: Vec<TemperatureStat> = flattened
        .map(|sensor| TemperatureStat {
            sensor_key: format_sensor_key(&sensor).into(),
            temperature: sensor.current().celsius() as f32,
            high: sensor.high().map_or(0.0, |t| t.celsius()) as f32,
            critical: sensor.critical().map_or(0.0, |t| t.celsius()) as f32,
        })
        .collect();
    let stats_slice: gotypes::GoSlice = stats.as_slice().into();
    gotypes::go_slice_clone_into(out, &stats_slice, elem_type);
    true
}
