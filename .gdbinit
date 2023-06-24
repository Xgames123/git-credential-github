layout src
python
import gdb.printing

class RustCommandPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        cmd_str = self.val['program']['0']['0'].string()
        args = self.val['args']['0']
        args_len = args['1']['len']
        args_data = args['0']['ptr'].dereference().cast(gdb.lookup_type("const char").pointer())
        args_list = [str(args_data[i].string()) for i in range(args_len)]
        return 'Command {{ program: "{}", args: {} }}'.format(cmd_str, args_list)

def build_pretty_printer():
    pp = gdb.printing.RegexpCollectionPrettyPrinter("rust_printers")
    pp.add_printer('Command', '^std::process::Command$', RustCommandPrinter)
    return pp

gdb.printing.register_pretty_printer(None, build_pretty_printer())
end