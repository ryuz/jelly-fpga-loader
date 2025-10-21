use clap::{Parser, Subcommand};
use jelly_fpga_client::JellyFpgaClient;
use std::path::Path;
use anyhow::{Result, anyhow};

#[derive(Parser)]
#[command(name = "jelly-fpga-loader")]
#[command(about = "FPGA utility tool using jelly-fpga-client-rs")]
struct Cli {
    /// FPGA server IP address and port
    #[arg(short, long, default_value = "127.0.0.1:8051", global = true)]
    ip: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download bitstream to FPGA
    Bitdownload {
        /// Bitstream file path
        bitstream_file: String,
    },
    /// Apply DeviceTree Overlay
    Overlay {
        /// DTBO file path
        dtbo_file: String,
        /// Bitstream file to transfer with DTBO
        #[arg(short = 'b', long)]
        bit: Option<String>,
        /// Bin file to transfer with DTBO
        #[arg(long)]
        bin: Option<String>,
    },
    /// Register accelerator package
    RegisterAccel {
        /// Accelerator name
        accel_name: String,
        /// DTBO file path
        dtbo_file: String,
        /// Bitstream file path
        bitstream_file: String,
        /// JSON file path
        #[arg(short, long)]
        json: Option<String>,
    },
    /// Unregister accelerator package
    UnregisterAccel {
        /// Accelerator name
        accel_name: String,
    },
    /// Load accelerator package
    Load {
        /// Accelerator name
        accel_name: String,
    },
    /// Unload accelerator package
    Unload {
        /// Slot number (default: 0)
        #[arg(default_value = "0")]
        slot: i32,
    },
    /// Convert DTS file to DTBO file
    Dts2dtbo {
        /// Input DTS file path
        dts_file: String,
        /// Output DTBO file path
        dtbo_file: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Connect to FPGA server
    let server_addr = format!("http://{}", cli.ip);
    let mut client = JellyFpgaClient::connect(server_addr).await
        .map_err(|e| anyhow!("Failed to connect to FPGA server: {}", e))?;
    
    match cli.command {
        Commands::Bitdownload { bitstream_file } => {
            bitdownload(&mut client, &bitstream_file).await?;
        },
        Commands::Overlay { dtbo_file, bit, bin } => {
            overlay(&mut client, &dtbo_file, bit.as_deref(), bin.as_deref()).await?;
        },
        Commands::RegisterAccel { accel_name, dtbo_file, bitstream_file, json } => {
            register_accel(&mut client, &accel_name, &bitstream_file, &dtbo_file, json.as_deref()).await?;
        },
        Commands::UnregisterAccel { accel_name } => {
            unregister_accel(&mut client, &accel_name).await?;
        },
        Commands::Load { accel_name } => {
            load(&mut client, &accel_name).await?;
        },
        Commands::Unload { slot } => {
            unload(&mut client, slot).await?;
        },
        Commands::Dts2dtbo { dts_file, dtbo_file } => {
            dts2dtbo(&mut client, &dts_file, &dtbo_file).await?;
        },
    }
    
    Ok(())
}

async fn bitdownload(client: &mut JellyFpgaClient, bitstream_file: &str) -> Result<()> {
    println!("Downloading bitstream: {}", bitstream_file);
    
    // Extract filename for the firmware name
    let filename = Path::new(bitstream_file)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid bitstream filename"))?;
    
    // Upload firmware file
    client.upload_firmware_file(filename, bitstream_file).await
        .map_err(|e| anyhow!("Failed to upload firmware: {}", e))?;
    
    // Load bitstream
    client.load_bitstream(filename).await
        .map_err(|e| anyhow!("Failed to load bitstream: {}", e))?;
    
    // Clean up uploaded files after completion
    client.remove_firmware(filename).await
        .map_err(|e| anyhow!("Failed to delete bitstream file from firmware: {}", e))?;
    
    println!("Bitstream downloaded successfully");
    Ok(())
}

async fn overlay(client: &mut JellyFpgaClient, dtbo_file: &str, bit_file: Option<&str>, bin_file: Option<&str>) -> Result<()> {
    println!("Applying DeviceTree Overlay: {}", dtbo_file);
    
    let mut uploaded_files = Vec::new(); // Track uploaded files for cleanup
    
    // Handle bitstream/bin file upload
    if let Some(bit_file) = bit_file {
        // Upload bitstream and convert to bin
        let bitstream_name = Path::new(bit_file)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid bitstream filename"))?;
        
        println!("Uploading bitstream: {}", bit_file);
        client.upload_firmware_file(bitstream_name, bit_file).await
            .map_err(|e| anyhow!("Failed to upload bitstream: {}", e))?;
        uploaded_files.push(bitstream_name.to_string());
        
        // Convert to bin format
        let bin_name = format!("{}.bin", bitstream_name);
        println!("Converting bitstream to bin format: {}", bin_name);
        client.bitstream_to_bin(bitstream_name, &bin_name, "zynqmp").await
            .map_err(|e| anyhow!("Failed to convert bitstream to bin: {}", e))?;
        uploaded_files.push(bin_name);
    } else if let Some(bin_file) = bin_file {
        // Upload bin file directly
        let bin_name = Path::new(bin_file)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid bin filename"))?;
        
        println!("Uploading bin file: {}", bin_file);
        client.upload_firmware_file(bin_name, bin_file).await
            .map_err(|e| anyhow!("Failed to upload bin file: {}", e))?;
        uploaded_files.push(bin_name.to_string());
    }
    
    // Handle DTS to DTBO conversion if needed
    let dtbo_name = if dtbo_file.ends_with(".dts") {
        println!("Converting DTS to DTBO...");
        let dts_content = std::fs::read_to_string(dtbo_file)
            .map_err(|e| anyhow!("Failed to read DTS file: {}", e))?;
        
        let (success, dtb_data) = client.dts_to_dtb(&dts_content).await
            .map_err(|e| anyhow!("Failed to convert DTS to DTB: {}", e))?;
        
        if !success {
            return Err(anyhow!("DTS to DTB conversion failed"));
        }
        
        // Upload the converted DTBO with .dtbo extension
        let base_name = Path::new(dtbo_file)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid DTS filename"))?;
        let dtbo_name = format!("{}.dtbo", base_name);
        
        client.upload_firmware(&dtbo_name, dtb_data).await
            .map_err(|e| anyhow!("Failed to upload converted DTBO: {}", e))?;
        uploaded_files.push(dtbo_name.clone());
        
        dtbo_name
    } else {
        // Upload DTBO file directly with original filename
        let dtbo_name = Path::new(dtbo_file)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid DTBO filename"))?;
        
        if !client.upload_firmware_file(dtbo_name, dtbo_file).await
            .map_err(|e| anyhow!("Failed to upload DTBO: {}", e))? {
            return Err(anyhow!("Failed to upload DTBO file"));
        }

        uploaded_files.push(dtbo_name.to_string());
        
        dtbo_name.to_string()
    };
    
    // Load DTBO
    if !client.load_dtbo(&dtbo_name).await
        .map_err(|e| anyhow!("Failed to load DTBO: {}", e))? {
        return Err(anyhow!("Failed to apply DeviceTree Overlay"));
    }

    // Clean up uploaded files after completion
    for filename in uploaded_files {
        client.remove_firmware(&filename).await
            .map_err(|e| anyhow!("Failed to delete file '{}' from firmware: {}", filename, e))?;
    }
    
    println!("DeviceTree Overlay applied successfully");
    Ok(())
}

async fn register_accel(
    client: &mut JellyFpgaClient,
    accel_name: &str,
    bitstream_file: &str,
    dtbo_file: &str,
    json_file: Option<&str>
) -> Result<()> {
    println!("Registering accelerator: {}", accel_name);
    
    // Upload bitstream file and convert to bin if needed
    let bin_name = if bitstream_file.ends_with(".bit") {
        println!("Converting bitstream to bin format...");
        let bitstream_name = Path::new(bitstream_file)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid bitstream filename"))?;
        
        // Upload bitstream first
        if !client.upload_firmware_file(bitstream_name, bitstream_file).await
            .map_err(|e| anyhow!("Failed to upload bitstream: {}", e))? {
            return Err(anyhow!("Failed to upload bitstream file"));
        }
        
        // Convert to bin
        let bin_name = format!("{}.bin", bitstream_name);
        client.bitstream_to_bin(bitstream_name, &bin_name, "zynqmp").await // Assuming zynqmp for KV260
            .map_err(|e| anyhow!("Failed to convert bitstream to bin: {}", e))?;
        
        bin_name
    } else {
        // Upload bin file directly
        let bin_name = Path::new(bitstream_file)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid bin filename"))?;
        
        if !client.upload_firmware_file(bin_name, bitstream_file).await
            .map_err(|e| anyhow!("Failed to upload bin file: {}", e))? {
            return Err(anyhow!("Failed to upload bin file"));
        }
        
        bin_name.to_string()
    };
    
    // Handle DTBO file
    let dtbo_name = if dtbo_file.ends_with(".dts") {
        println!("Converting DTS to DTBO...");
        let dts_content = std::fs::read_to_string(dtbo_file)
            .map_err(|e| anyhow!("Failed to read DTS file: {}", e))?;
        
        let (success, dtb_data) = client.dts_to_dtb(&dts_content).await
            .map_err(|e| anyhow!("Failed to convert DTS to DTB: {}", e))?;
        
        if !success {
            return Err(anyhow!("DTS to DTB conversion failed"));
        }
        
        // Upload the converted DTBO with .dtbo extension
        let base_name = Path::new(dtbo_file)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid DTS filename"))?;
        let dtbo_name = format!("{}.dtbo", base_name);
        
        client.upload_firmware(&dtbo_name, dtb_data).await
            .map_err(|e| anyhow!("Failed to upload converted DTBO: {}", e))?;
        
        dtbo_name
    } else {
        let dtbo_name = Path::new(dtbo_file)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid DTBO filename"))?;
        
        if !client.upload_firmware_file(dtbo_name, dtbo_file).await
            .map_err(|e| anyhow!("Failed to upload DTBO: {}", e))? {
            return Err(anyhow!("Failed to upload DTBO file"));
        }

        dtbo_name.to_string()
    };
    
    // Upload JSON file if provided
    let json_name = if let Some(json_file) = json_file {
        let json_name = Path::new(json_file)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid JSON filename"))?;
        
        if !client.upload_firmware_file(json_name, json_file).await
            .map_err(|e| anyhow!("Failed to upload JSON file: {}", e))? {
            return Err(anyhow!("Failed to upload JSON file"));
        }
        
        Some(json_name)
    } else {
        None
    };
    
    // Register accelerator
    println!("accel_name = {}, bin_name = {}, dtbo_name = {}, json_name = {:?}", accel_name, bin_name, dtbo_name, json_name);
    client.register_accel(accel_name, &bin_name, &dtbo_name, json_name, true).await
        .map_err(|e| anyhow!("Failed to register accelerator: {}", e))?;
    
    // Clean up uploaded files after registration
    if bitstream_file.ends_with(".bit") {
        let bitstream_name = Path::new(bitstream_file)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid bitstream filename"))?;
        client.remove_firmware(bitstream_name).await
            .map_err(|e| anyhow!("Failed to delete bitstream file from firmware: {}", e))?;
    }
    client.remove_firmware(&bin_name).await
        .map_err(|e| anyhow!("Failed to delete bin file from firmware: {}", e))?;
    client.remove_firmware(&dtbo_name).await
        .map_err(|e| anyhow!("Failed to delete dtbo file from firmware: {}", e))?;
    if let Some(json_name) = json_name {
        client.remove_firmware(json_name).await
            .map_err(|e| anyhow!("Failed to delete json file from firmware: {}", e))?;
    }

    println!("Accelerator registered successfully");
    Ok(())
}

async fn unregister_accel(client: &mut JellyFpgaClient, accel_name: &str) -> Result<()> {
    println!("Unregistering accelerator: {}", accel_name);
    
    client.unregister_accel(accel_name).await
        .map_err(|e| anyhow!("Failed to unregister accelerator: {}", e))?;
    
    println!("Accelerator unregistered successfully");
    Ok(())
}

async fn load(client: &mut JellyFpgaClient, accel_name: &str) -> Result<()> {
    println!("Loading accelerator: {}", accel_name);
    
    let (success, slot) = client.load(accel_name).await
        .map_err(|e| anyhow!("Failed to load accelerator: {}", e))?;
    
    if success {
        println!("Accelerator loaded successfully to slot {}", slot);
    } else {
        return Err(anyhow!("Failed to load accelerator"));
    }
    
    Ok(())
}

async fn unload(client: &mut JellyFpgaClient, slot: i32) -> Result<()> {
    println!("Unloading accelerator from slot: {}", slot);
    
    if !client.unload(slot).await
        .map_err(|e| anyhow!("Failed to unload accelerator: {}", e))? {
        return Err(anyhow!("Failed to unload accelerator from slot {}", slot));
    }
    
    println!("Accelerator unloaded successfully");
    Ok(())
}

async fn dts2dtbo(client: &mut JellyFpgaClient, dts_file: &str, dtbo_file: &str) -> Result<()> {
    println!("Converting DTS to DTBO: {} -> {}", dts_file, dtbo_file);
    
    // Read DTS file content
    let dts_content = std::fs::read_to_string(dts_file)
        .map_err(|e| anyhow!("Failed to read DTS file '{}': {}", dts_file, e))?;
    
    // Convert DTS to DTB using server
    let (success, dtb_data) = client.dts_to_dtb(&dts_content).await
        .map_err(|e| anyhow!("Failed to convert DTS to DTB: {}", e))?;
    
    if !success {
        return Err(anyhow!("DTS to DTB conversion failed"));
    }
    
    // Write DTB data to output file
    std::fs::write(dtbo_file, dtb_data)
        .map_err(|e| anyhow!("Failed to write DTBO file '{}': {}", dtbo_file, e))?;
    
    println!("DTS to DTBO conversion completed successfully");
    Ok(())
}
