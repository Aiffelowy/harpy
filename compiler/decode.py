#!/usr/bin/env python3
import sys;

def decode_bytecode(data):
    print("=== HEADER ===")
    
    # Magic number (5 bytes)
    magic_bytes = data[0:5]
    print(f"Magic: {magic_bytes} ({''.join(chr(b) for b in magic_bytes if 32 <= b <= 126)})")
    
    # Version (2 bytes)
    version = int.from_bytes(data[5:7], 'big')
    print(f"Version: 0x{version:04x}")
    
    # Flags (2 bytes)
    flags = int.from_bytes(data[7:9], 'big')
    print(f"Flags: 0x{flags:04x}")
    
    # Entry point (4 bytes)
    entry_point = int.from_bytes(data[9:13], 'big')
    print(f"Entry point: 0x{entry_point:04x}")
    
    # Offsets
    type_table_offset = int.from_bytes(data[13:17], 'big')
    const_pool_offset = int.from_bytes(data[17:21], 'big')
    function_table_offset = int.from_bytes(data[21:25], 'big')
    bytecode_offset = int.from_bytes(data[25:29], 'big')
    bytecode_size = int.from_bytes(data[29:33], 'big')
    
    print(f"Type table offset: {type_table_offset}")
    print(f"Const pool offset: {const_pool_offset}")
    print(f"Function table offset: {function_table_offset}")
    print(f"Bytecode offset: {bytecode_offset}")
    print(f"Bytecode size: {bytecode_size}")
    
    type_table = []

    pos = type_table_offset
    while pos < const_pool_offset:
        type_id = data[pos]
        if type_id == 0x01:
            prim_type_id = data[pos+1]
            size = data[pos+2]
            type_table.append({ "type": type_id, "primitive_id": prim_type_id, "size": size })
            pos += 3
        elif type_id == 0x00:
            type_table.append({ "type": type_id, "size": 0 })
            pos += 1
        elif type_id == 0x02:
            to = int.from_bytes(data[pos+1:pos+5], 'big')
            type_table.append({ "type": type_id, "size": 8, "points_to": to })
            pos += 5
        elif type_id == 0x03:
            to = int.from_bytes(data[pos+1:pos+5], 'big')
            type_table.append({ "type": type_id, "size": 8, "points_to": to })
            pos += 5
        elif type_id == 0x04:
            size = data[pos+1]
            type_table.append({ "type": type_id, "size": size })
            pos += 2
        else:
            print("BAD TYPE TABLE")
            exit(0)
    
    const_pool = []
    pos = const_pool_offset
    while pos < function_table_offset:
        type_id = int.from_bytes(data[pos:pos+4], 'big')
        pos += 4
        size = type_table[type_id]["size"];
        value = int.from_bytes(data[pos:pos+size], 'big')
        pos += size
        const_pool.append({ "type_id": type_id, "value": value })
    
    function_table = []
    pos = function_table_offset
    bytecode_start = function_table_offset + (len(data) - function_table_offset - bytecode_size)
    while pos < bytecode_start:
        code_offset = int.from_bytes(data[pos:pos+8], 'big')
        param_count = int.from_bytes(data[pos+8:pos+10], 'big')
        local_count = int.from_bytes(data[pos+10:pos+12], 'big')
        localss = []
        pos += 12
        for _ in range(local_count):
            local_type = int.from_bytes(data[pos:pos+4], 'big')
            localss.append(local_type)
            pos += 4

        function_table.append({ "offset": code_offset, "param_count": param_count, "local_cout": local_count, "locals": localss })
    
    # Define instruction parameter sizes
    instruction_info = {
        0x00: {"name": "NOP", "size": 1},
        0x01: {"name": "LOAD_CONST", "size": 5}, 
        0x10: {"name": "PUSH_ADDR_LOCAL", "size": 3}, 
        0x11: {"name": "LOAD_LOCAL", "size": 3}, 
        0x12: {"name": "STORE_LOCAL", "size": 3},
        0x31: {"name": "LOAD", "size": 1}, 
        0x32: {"name": "STORE", "size": 1}, 
        0x40: {"name": "BOX_ALLOC", "size": 5},
        0x50: {"name": "ADD", "size": 1}, 
        0x51: {"name": "SUB", "size": 1}, 
        0x52: {"name": "MUL", "size": 1}, 
        0x53: {"name": "DIV", "size": 1}, 
        0x54: {"name": "NEG", "size": 1}, 
        0x55: {"name": "INC", "size": 1},
        0x60: {"name": "JMP", "size": 9}, 
        0x61: {"name": "JMP_IF_TRUE", "size": 9}, 
        0x62: {"name": "JMP_IF_FALSE", "size": 9},
        0x70: {"name": "CALL", "size": 5}, 
        0x71: {"name": "RET", "size": 1},
        0x80: {"name": "EQ", "size": 1}, 
        0x81: {"name": "NEQ", "size": 1}, 
        0x82: {"name": "LT", "size": 1}, 
        0x83: {"name": "LTE", "size": 1}, 
        0x84: {"name": "GT", "size": 1}, 
        0x85: {"name": "GTE", "size": 1},
        0x86: {"name": "AND", "size": 1}, 
        0x87: {"name": "OR", "size": 1}, 
        0x88: {"name": "NOT", "size": 1},
        0x90: {"name": "POP", "size": 1}, 
        0x91: {"name": "DUP", "size": 1}, 
        0xFF: {"name": "HALT", "size": 1}
    }

    bytecode = []
    bytecode_start = len(data) - bytecode_size
    pos = bytecode_start
    while pos < len(data):
        opcode = data[pos]
        
        info = instruction_info.get(opcode, {"name": f"UNKNOWN_{opcode:02x}", "size": 1})
        name = info["name"]
        size = info["size"]
        
        if opcode == 0x01:  # LOAD_CONST
            const_idx = int.from_bytes(data[pos+1:pos+5], 'big')
            bytecode.append({"opcode": name, "param": const_idx, "size": size, "byte_offset": pos - bytecode_start})
        elif opcode in [0x10, 0x11, 0x12]:  # LOCAL operations
            local_addr = int.from_bytes(data[pos+1:pos+3], 'big')
            bytecode.append({"opcode": name, "param": local_addr, "size": size, "byte_offset": pos - bytecode_start})
        elif opcode == 0x40:
            type_id = int.from_bytes(data[pos+1:pos+5], 'big')
            bytecode.append({ "opcode": name, "param": type_id, "size": size, "byte_offset": pos - bytecode_start })
        elif opcode in [0x60, 0x61, 0x62]:  # JMP operations
            target = int.from_bytes(data[pos+1:pos+9], 'big')
            bytecode.append({"opcode": name, "param": target, "size": size, "byte_offset": pos - bytecode_start})
        elif opcode == 0x70:  # CALL
            func_idx = int.from_bytes(data[pos+1:pos+5], 'big')
            bytecode.append({"opcode": name, "param": func_idx, "size": size, "byte_offset": pos - bytecode_start})
        else:
            bytecode.append({"opcode": name, "param": None, "size": size, "byte_offset": pos - bytecode_start})
        
        pos += size

    return type_table, const_pool, function_table, bytecode

def print_type_table(tt):
    print(f"\n=== TYPE TABLE ===")
    for i, ty in enumerate(tt):
        idx = ty["type"]
        size = ty["size"]
        
        type_name = {
            0x00: "Void",
            0x01: "Primitive", 
            0x02: "Boxed",
            0x03: "Reference",
            0x04: "Custom"
        }.get(idx, f"Unknown(0x{idx:02x})")
        
        print(f"Type {i}: {type_name}, Size: {size} bytes", end="")
        
        if "primitive_id" in ty:
            prim_name = {
                0x01: "int",
                0x02: "float", 
                0x03: "str",
                0x04: "bool"
            }.get(ty["primitive_id"], f"Unknown(0x{ty['primitive_id']:02x})")
            print(f", Primitive: {prim_name}")
        elif "points_to" in ty:
            pt = ty["points_to"]
            print(f", Points to: {pt}")
        else:
            print()

def print_const_pool(cp, tt):
    print(f"\n=== CONST POOL ===")
    for i, const in enumerate(cp):
        type_id = const["type_id"]
        value = const["value"]
        print(f"Const {i}: Type {type_id}, Value: {value}")

def print_function_table(ft):
    print(f"\n=== FUNCTION TABLE ===")
    for i, func in enumerate(ft):
        offset = func["offset"]
        param_count = func["param_count"]
        local_count = func["local_cout"]  # Note: typo in original code
        locals_types = func["locals"]
        print(f"Function {i}:")
        print(f"    Code offset: 0x{offset:04x}")
        print(f"    Param count: {param_count}")
        print(f"    Local count: {local_count}")
        if locals_types:
            print(f"    Local types: {locals_types}")

def print_bytecode(bc):
    print(f"\n=== BYTECODE ===")
    for instr in bc:
        opcode = instr["opcode"]
        param = instr["param"]
        byte_offset = instr["byte_offset"]
        
        if param is not None:
            # Print jump targets as hex (code addresses), others as decimal
            if opcode in ["JMP", "JMP_IF_TRUE", "JMP_IF_FALSE"]:
                print(f"[0x{byte_offset:04x}] {opcode} 0x{param:04x}")
            else:
                print(f"[0x{byte_offset:04x}] {opcode} {param}")
        else:
            print(f"[0x{byte_offset:04x}] {opcode}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python3 decode.py <bytecode_file>")
        sys.exit(1)
        
    file = open(sys.argv[1], "rb")
    data = file.read()
    file.close()
    
    tt, cp, ft, bc = decode_bytecode(data)
    
    print_type_table(tt)
    print()
    print_const_pool(cp, tt)
    print()
    print_function_table(ft)
    
    print_bytecode(bc)




