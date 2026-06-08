-- Trace SNES APUI writes and SPC DSP register writes in Mesen2.
--
-- Output defaults to external/mesen2-oracle/mesen2-apui-dsp-trace.jsonl.
-- Override with:
--   M2_TRACE_OUT=/path/to/trace.jsonl
--   M2_TRACE_FRAMES=180

local default_output = "external/mesen2-oracle/mesen2-apui-dsp-trace.jsonl"
local default_frames = 180

local function getenv(name)
  if os and os.getenv then
    local ok, value = pcall(os.getenv, name)
    if ok then
      return value
    end
  end
  return nil
end

local output_path = getenv("M2_TRACE_OUT") or default_output
local frame_limit = tonumber(getenv("M2_TRACE_FRAMES") or "") or default_frames
local pixel_x = tonumber(getenv("M2_TRACE_PIXEL_X") or "")
local pixel_y = tonumber(getenv("M2_TRACE_PIXEL_Y") or "")
local cgram_start = tonumber(getenv("M2_TRACE_CGRAM_START") or "")
local cgram_count = tonumber(getenv("M2_TRACE_CGRAM_COUNT") or "") or 0
local trace_spc_vars = getenv("M2_TRACE_SPC_VARS") == "1"

local out = assert(io.open(output_path, "w"))
local frame = 0
local spc_dsp_addr = 0
local event_count = 0

local function hex(value, width)
  return string.format("%0" .. tostring(width) .. "x", value or 0)
end

local function json_string(value)
  value = tostring(value or "")
  value = value:gsub("\\", "\\\\"):gsub("\"", "\\\"")
  value = value:gsub("\n", "\\n"):gsub("\r", "\\r"):gsub("\t", "\\t")
  return "\"" .. value .. "\""
end

local function emit(fields)
  event_count = event_count + 1
  fields.event = event_count
  fields.frame = fields.frame or frame

  local ordered = {
    "event",
    "frame",
    "kind",
    "cpu",
    "addr",
    "value",
    "dsp_reg",
    "note",
  }
  local seen = {}
  local parts = {}

  for _, key in ipairs(ordered) do
    local value = fields[key]
    if value ~= nil then
      seen[key] = true
      if type(value) == "number" then
        table.insert(parts, json_string(key) .. ":" .. tostring(value))
      else
        table.insert(parts, json_string(key) .. ":" .. json_string(value))
      end
    end
  end

  for key, value in pairs(fields) do
    if not seen[key] then
      if type(value) == "number" then
        table.insert(parts, json_string(key) .. ":" .. tostring(value))
      else
        table.insert(parts, json_string(key) .. ":" .. json_string(value))
      end
    end
  end

  out:write("{" .. table.concat(parts, ",") .. "}\n")
  out:flush()
end

local function safe_add_memory_callback(name, callback, callback_type, start_addr, end_addr, cpu_type, mem_type)
  local ok, result = pcall(function()
    return emu.addMemoryCallback(callback, callback_type, start_addr, end_addr, cpu_type, mem_type)
  end)
  if not ok then
    emit({
      kind = "callback_error",
      note = name .. ": " .. tostring(result),
    })
  end
  return ok, result
end

local function safe_add_event_callback(name, callback, event_type)
  local ok, result = pcall(function()
    return emu.addEventCallback(callback, event_type)
  end)
  if not ok then
    emit({
      kind = "callback_error",
      note = name .. ": " .. tostring(result),
    })
  end
  return ok, result
end

local function read_spc(addr)
  local ok, value = pcall(function()
    return emu.read(addr, emu.memType.spcMemory, false)
  end)
  if ok then
    return value
  end
  return nil
end

local function add_spc_vars(fields)
  if not trace_spc_vars then
    return fields
  end
  fields.spc_new3 = read_spc(0x0003)
  fields.spc_last3 = read_spc(0x000b)
  fields.spc_sfx_timer = read_spc(0x0043)
  fields.spc_key_on = read_spc(0x0045)
  fields.spc_key_off = read_spc(0x0046)
  fields.spc_is_chan_on = read_spc(0x001a)
  fields.spc_port3_active = read_spc(0x03d1)
  fields.spc_ch7_countdown = read_spc(0x03af)
  return fields
end

emit({
  kind = "trace_start",
  note = "Mesen2 APUI/DSP trace; addr/value fields are decimal, use addr_hex/value_hex for display.",
  output = output_path,
  frame_limit = frame_limit,
})

safe_add_event_callback("start_frame", function(cpu_type)
  emit(add_spc_vars({
    kind = "start_frame",
    cpu = tostring(cpu_type),
  }))
end, emu.eventType.startFrame)

safe_add_event_callback("nmi", function(cpu_type)
  emit({
    kind = "nmi",
    cpu = tostring(cpu_type),
  })
end, emu.eventType.nmi)

safe_add_event_callback("end_frame", function(cpu_type)
  if cgram_start ~= nil and cgram_count > 0 then
    local values = {}
    for i = 0, cgram_count - 1 do
      local addr = (cgram_start + i) * 2
      local ok, lo = pcall(function()
        return emu.read(addr, emu.memType.snesCgRam, false)
      end)
      local ok2, hi = pcall(function()
        return emu.read(addr + 1, emu.memType.snesCgRam, false)
      end)
      if ok and ok2 then
        table.insert(values, hex((hi * 256 + lo) % 0x8000, 4))
      else
        table.insert(values, "err")
      end
    end
    emit({
      kind = "cgram",
      start = cgram_start,
      count = cgram_count,
      values = table.concat(values, ","),
    })
  end

  if pixel_x ~= nil and pixel_y ~= nil then
    local ok, argb = pcall(function()
      return emu.getPixel(pixel_x, pixel_y)
    end)
    if ok then
      local a = math.floor(argb / 0x1000000) % 0x100
      local r = math.floor(argb / 0x10000) % 0x100
      local g = math.floor(argb / 0x100) % 0x100
      local b = argb % 0x100
      emit({
        kind = "pixel",
        x = pixel_x,
        y = pixel_y,
        argb = argb,
        a = a,
        r = r,
        g = g,
        b = b,
        rgb_hex = "$" .. hex(r, 2) .. hex(g, 2) .. hex(b, 2),
      })
    else
      emit({
        kind = "pixel_error",
        x = pixel_x,
        y = pixel_y,
        note = tostring(argb),
      })
    end
  end
  emit(add_spc_vars({
    kind = "end_frame",
    cpu = tostring(cpu_type),
  }))
  frame = frame + 1
  if frame >= frame_limit then
    emit({
      kind = "stop",
      note = "frame limit reached",
    })
    out:close()
    emu.stop(0)
  end
end, emu.eventType.endFrame)

safe_add_memory_callback("s_cpu_apui_write", function(address, value)
  emit(add_spc_vars({
    kind = "apui_write",
    cpu = "snes",
    addr = address,
    value = value,
    addr_hex = "$" .. hex(address, 4),
    value_hex = "$" .. hex(value, 2),
  }))
end, emu.callbackType.write, 0x2140, 0x2143, emu.cpuType.snes, emu.memType.snesMemory)

safe_add_memory_callback("s_cpu_inidisp_write", function(address, value)
  emit({
    kind = "ppu_write",
    cpu = "snes",
    addr = address,
    value = value,
    addr_hex = "$" .. hex(address, 4),
    value_hex = "$" .. hex(value, 2),
    reg = "INIDISP",
    brightness = value % 16,
    forced_blank = value >= 128 and 1 or 0,
  })
end, emu.callbackType.write, 0x2100, 0x2100, emu.cpuType.snes, emu.memType.snesMemory)

safe_add_memory_callback("s_cpu_startup_state_write", function(address, value)
  local name = "wram"
  if address == 0x0010 then
    name = "main_module_index"
  elseif address == 0x0011 then
    name = "submodule_index"
  elseif address == 0x0013 then
    name = "inidisp_copy"
  elseif address == 0x00b0 then
    name = "subsubmodule_index"
  end
  emit({
    kind = "wram_write",
    cpu = "snes",
    addr = address,
    value = value,
    addr_hex = "$" .. hex(address, 4),
    value_hex = "$" .. hex(value, 2),
    name = name,
  })
end, emu.callbackType.write, 0x0010, 0x00b0, emu.cpuType.snes, emu.memType.snesMemory)

safe_add_memory_callback("spc_dsp_addr_write", function(address, value)
  spc_dsp_addr = value
  emit({
    kind = "spc_dsp_addr",
    cpu = "spc",
    addr = address,
    value = value,
    addr_hex = "$" .. hex(address, 4),
    value_hex = "$" .. hex(value, 2),
  })
end, emu.callbackType.write, 0x00f2, 0x00f2, emu.cpuType.spc, emu.memType.spcMemory)

safe_add_memory_callback("spc_dsp_data_write", function(address, value)
  emit(add_spc_vars({
    kind = "spc_dsp_data",
    cpu = "spc",
    addr = address,
    value = value,
    dsp_reg = spc_dsp_addr,
    addr_hex = "$" .. hex(address, 4),
    value_hex = "$" .. hex(value, 2),
    dsp_reg_hex = "$" .. hex(spc_dsp_addr, 2),
  }))
end, emu.callbackType.write, 0x00f3, 0x00f3, emu.cpuType.spc, emu.memType.spcMemory)
