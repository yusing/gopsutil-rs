package main

import (
	"context"
	"fmt"
	"log"
	"runtime"
	"time"

	"github.com/yusing/gopsutil-rs/pkg/gopsutil"
)

func main() {
	fmt.Printf("gopsutil-rs example - %s/%s\n\n", runtime.GOOS, runtime.GOARCH)

	// Create library instance
	lib, err := gopsutil.New()
	if err != nil {
		log.Fatalf("Failed to load gopsutil library: %v", err)
	}
	defer lib.Close()

	// Display system information
	cpu, err := lib.GetCPUPercent(context.Background(), 500*time.Millisecond)
	if err != nil {
		log.Fatalf("Failed to get CPU percent: %v", err)
	}

	memory, err := lib.GetMemoryInfo()
	if err != nil {
		log.Fatalf("Failed to get memory info: %v", err)
	}

	disk, err := lib.GetDiskUsageByPartition()
	if err != nil {
		log.Fatalf("Failed to get disk usage: %v", err)
	}

	diskIO, err := lib.GetDiskIOByPartition()
	if err != nil {
		log.Fatalf("Failed to get disk IO: %v", err)
	}

	network, err := lib.GetNetworkInfo()
	if err != nil {
		log.Fatalf("Failed to get network info: %v", err)
	}

	temperatures, err := lib.GetTemperatures()
	if err != nil {
		log.Fatalf("Failed to get temperatures: %v", err)
	}

	fmt.Printf("System Information:\n")
	fmt.Printf("  CPU Usage:      %.2f%%\n", cpu)
	fmt.Printf("  Memory Total:   %s\n", formatBytes(memory.Total))
	fmt.Printf("  Memory Used:    %s (%.2f%%)\n",
		formatBytes(memory.Used), memory.UsedPercent)
	fmt.Printf("  Memory Available: %s\n", formatBytes(memory.Available))
	fmt.Printf("  Network Sent: %s\n", formatBytes(network.BytesSent))
	fmt.Printf("  Network Received: %s\n", formatBytes(network.BytesRecv))
	fmt.Printf("  Network Upload Speed: %s\n", formatBytes(uint64(network.UploadSpeed)))
	fmt.Printf("  Network Download Speed: %s\n", formatBytes(uint64(network.DownloadSpeed)))
	for _, temperature := range temperatures {
		fmt.Printf("  Temperature %s: %.2f°C (High: %.2f°C, Critical: %.2f°C)\n", temperature.SensorKey, temperature.Temperature, temperature.High, temperature.Critical)
	}
	for name, usage := range disk {
		fmt.Printf("  Disk Usage %s:\n", name)
		fmt.Printf("    Device: %s\n", usage.Device)
		fmt.Printf("    Path: %s\n", usage.Path)
		fmt.Printf("    Fs: %s\n", usage.Fs)
		fmt.Printf("    Total: %s\n", formatBytes(usage.Total))
		fmt.Printf("    Used: %s (%.2f%%)\n", formatBytes(usage.Used), usage.UsedPercent)
		fmt.Printf("    Free: %s\n", formatBytes(usage.Free))
	}
	for name, io := range diskIO {
		fmt.Printf("  Disk IO %s:\n", name)
		fmt.Printf("    Read Bytes: %s\n", formatBytes(io.ReadBytes))
		fmt.Printf("    Write Bytes: %s\n", formatBytes(io.WriteBytes))
		fmt.Printf("    Read Count: %d\n", io.ReadCount)
		fmt.Printf("    Write Count: %d\n", io.WriteCount)
		fmt.Printf("    Iops: %d\n", io.Iops)
		fmt.Printf("    Read Speed: %s\n", formatBytes(uint64(io.ReadSpeed)))
		fmt.Printf("    Write Speed: %s\n", formatBytes(uint64(io.WriteSpeed)))
	}
}

// formatBytes converts bytes to human readable format
func formatBytes(bytes uint64) string {
	const unit = 1024
	if bytes < unit {
		return fmt.Sprintf("%d B", bytes)
	}
	div, exp := uint64(unit), 0
	for n := bytes / unit; n >= unit; n /= unit {
		div *= unit
		exp++
	}
	return fmt.Sprintf("%.1f %cB", float64(bytes)/float64(div), "KMGTPE"[exp])
}
