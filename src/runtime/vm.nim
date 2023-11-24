import ../compile/codegen_def
import print
import vm_def
import ../etc/utils
import strutils
import ../etc/enviroments
import tables
# template to quickly add binary operations
template BIN_OP(tasks: untyped): untyped =
  var reg0_addr {.inject.} = system.int(bytecode[vm.ip])
  var reg1_addr {.inject.} = bytecode[vm.ip + 1] 
  vm.checkRegs(reg1_addr)
  var left {.inject.}:ValueType = vm.reg[reg0_addr].kind
  var right {.inject.}: ValueType = vm.reg[reg1_addr].kind
  var reg0 {.inject.}: ptr REG = addr vm.reg[reg0_addr]
  var reg1 {.inject.}: ptr REG = addr vm.reg[reg1_addr]
  tasks
  vm.changeCond(reg0_addr) 
  vm.ip += 2
  
proc interpret*(bytecode: seq[byte]): VM =
  var vm = VM()
  vm.ip = 0
  print bytecode

  var env = Enviroment(varibles: Table[uint16, RuntimeValue]()) 
  while vm.ip < bytecode.len:
    var op = OP(bytecode[vm.ip])    
    print op
    vm.ip += 1
    case op: 
      of OP_CONSTS:
        var consts_count = makeInt(bytecode[vm.ip..vm.ip + 1])
        vm.ip += 2
        print consts_count
    
        for i in 0 .. consts_count:
          print i
          var tag = OP(bytecode[vm.ip])
          vm.ip += 1
          case tag:
            of TAG_INT:
              var bytes = bytecode[vm.ip .. vm.ip + 3]
              var int_val = RuntimeValue(kind: int, bytes: bytes)
              print bytes
              vm.consts.add(int_val)
              vm.ip += 4
            of TAG_STR:
              var count = system.int(bytecode[vm.ip..vm.ip + 1].makeInt())
              var bytes = bytecode[(vm.ip)..(vm.ip + count + 1)]
              vm.ip += 2 + count
              var str_val = RuntimeValue(kind: str, bytes: bytes)
              vm.consts.add(str_val)
            else:
              echo "ERROR while loading consts unknown type " & $tag & " please report this!"
              vm.results = UNKNOWN_OP
              vm.results_eval = "INVAILD TAG " & $tag
              return vm
      of OP_STRNAME:
        var count = makeInt(bytecode[vm.ip..vm.ip + 1])

        var regip = bytecode[vm.ip]
        vm.checkRegs(regip)
        var reg = addr vm.reg[regip]

        vm.ip += 3

        env.setVar(uint16(count), RuntimeValue(kind: reg.kind, bytes: reg.bytes))
      of OP_LOADNAME:
        var regip = bytecode[vm.ip]
        vm.checkRegs(regip)
        var reg = addr vm.reg[regip]
        var index = uint16(makeInt(bytecode[vm.ip + 1..vm.ip + 2]))
        vm.ip += 3
        
        var val = env.getVarVal(index)
        reg.bytes = val.bytes
        reg.kind = val.kind
    
      of OP_LOAD_CONST:
        var reg0 = bytecode[vm.ip]
        var imm = makeInt(bytecode[vm.ip + 1..vm.ip + 2])
        vm.ip += 3
    
        vm.checkRegs(reg0)
        var consts = vm.consts[imm - 1] 
      
        vm.reg[reg0] = REG(kind: consts.kind, bytes: consts.bytes)
        vm.changeCond(system.int(reg0))
      of OP_LOAD:
        var reg0 = bytecode[vm.ip]
        print reg0
        var imm = bytecode[vm.ip + 1] 
        print imm
        vm.ip += 2
        vm.checkRegs(reg0)
        
        vm.reg[reg0].bytes = @[imm]
        vm.changeCond(system.int(reg0))
        print vm.reg
        
      of OP_ADD:
        BIN_OP:
          case left:
            of int:
              reg0.bytes = (makeInt(reg0.bytes) + makeInt(reg1.bytes)).to4Bytes()  
            of str: 
              var right_bytes = reg1.bytes
              if right == ValueType.int:
                 print reg1.bytes
                 right_bytes = ($makeInt(reg1.bytes)).StrToBytes 
              reg0.bytes = reg0.bytes & right_bytes
            else:
              discard
      of OP_SUB:
        BIN_OP:
          case left:
            of int:
              reg0.bytes = (makeInt(reg0.bytes) - makeInt(reg1.bytes)).to4Bytes()          
            of str:
              reg0.bytes = (BytesToStr(reg0.bytes).replace(BytesToStr(reg1.bytes), "")).StrToBytes
            else:
              discard
      of OP_MUL:
        BIN_OP:
          case left:
            of int:
              reg0.bytes = (makeInt(reg0.bytes) * makeInt(reg1.bytes)).to4Bytes
            else:
              discard
      of OP_DIV:
        BIN_OP:
          case left:
            of int:
              reg0.bytes = uint32(system.int(makeInt(reg0.bytes)) / system.int(makeInt(reg1.bytes))).to4Bytes    
            else:
              discard 
      else: 
        echo "ERROR while executing: invaild insturaction please report this! " & $op
        vm.results = UNKNOWN_OP
        vm.results_eval = "INVAILD " & $op 
        return vm
  
  return vm
