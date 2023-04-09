// ** UNDER DEVELOPMENT **

use std::collections::HashMap;

fn asm_static_buf() {
    println!(
        "; bss
section '.bss' writable
    head:      rb 1
    state:     rb 1
    tape:      rb 256
    print_buf: rb 256
        "
    )
}

fn write_to_static_buf(addr: &str, val: &[char]) {
    println!("    ; write_to_static_buf");
    for (i, v) in val.iter().enumerate() {
        println!("    mov byte [{addr} + {i}], '{v}'");
    }
}

fn asm_exit() {
    println!(
        "    ; exit
    mov rax, 60
    mov rdi, 0
    syscall"
    );
}

#[derive(Default)]
pub struct Compiler<'c> {
    symbols: HashMap<&'c str, &'c str>,
}
impl<'c> Compiler<'c> {
    fn asm_print(&self, asm_str: &str) {
        let s = self.symbols.get(asm_str).unwrap();
        println!(
            "    ; print
    mov rdx, {}		; length to rdx
    mov rsi, str_{}	; address to rsi
    mov	rdi, 1		; stdout
    mov rax, 1		; write syscall
    syscall
        ",
            asm_str.len(),
            s
        );
    }

    fn asm_print_addr(&self, addr: &str, len: usize) {
        println!(
            "    ; print_addr
    mov rdx, {len}  ; length to rdx
    mov rsi, {addr} 	; address to rsi
    mov	rdi, 1		; stdout
    mov rax, 1		; write syscall
    syscall
        ",
        );
    }

    fn asm_str(&mut self, str: &'c str) {
        self.asm_str_name(str, str)
    }

    fn asm_str_name(&mut self, str: &'c str, name: &'c str) {
        let s = str
            .as_bytes()
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        self.symbols.entry(str).or_insert_with(|| {
            println!("    str_{}: db {}", name, s);
            name
        });
    }

    fn tape_print(&self, len: usize) {
        println!(
            "
; tape_print
tape_print:
    push rax
    push rcx
    mov byte [print_buf], '['
    mov byte [print_buf+1], ' '
        "
        );

        for tape_i in 0..len {
            let pi1 = (tape_i + 1) * 2;
            let pi2 = pi1 + 1;
            println!(
                "    mov rax, [tape+{tape_i}]
    mov [print_buf+{pi1}], rax
    mov byte [print_buf+{pi2}], ' '
    ",
            );
        }

        println!(
            "    mov byte [print_buf+{ket}], ']'
    mov byte [print_buf+{nl}], 10",
            ket = (len + 1) * 2,
            nl = (len + 1) * 2 + 1,
        );
        // [ 1 2 3 4 ]
        //   ^

        println!(
            "
    mov ecx, head
    mov rax, {after_nl}
l1:
    mov byte [print_buf+rax], ' '
    inc rax
    loop l1

    mov byte [print_buf+rax], '^'
    pop rax
    pop rcx
    ret
        ",
            after_nl = (len + 1) * 2 + 2
        );
    }

    pub fn compile_program(&mut self, program: crate::parser::Program<'c>) {
        println!("format ELF64");
        println!("section '.data' writeable");
        self.asm_str_name("[", "bra");
        self.asm_str_name("]", "ket");
        self.asm_str_name("->", "rightarrow");
        self.asm_str_name("<-", "leftarrow");
        self.asm_str_name("^", "halt");
        self.asm_str_name(" ", "space");
        self.asm_str_name("\n", "nl");
        self.asm_str_name(" -- HALT -- ", "halted");

        for instr in program.program.iter() {
            self.asm_str(instr.next_state);
            self.asm_str(instr.read);
            self.asm_str(instr.write);
        }

        for e in program.runs[0].tape.iter() {
            self.asm_str(e);
        }

        asm_static_buf(); // print_buf & head & tape & state

        println!("section '.text' executable");
        println!("public _start");

        println!("_start:");
        println!("    mov byte [head], 3");

        assert!(program.runs.len() == 1);
        let run = &program.runs[0];
        assert!(
            run.tape.iter().all(|s| s.len() == 1),
            "only one char symbols are support for asm target"
        );

        let tape = run
            .tape
            .iter()
            .map(|c| c.chars().next().unwrap())
            .collect::<Vec<char>>();

        write_to_static_buf("tape", &tape);
        self.asm_print_addr("tape", tape.len());
        self.asm_print("\n");

        println!("    call tape_print");
        self.asm_print_addr("print_buf",  ((tape.len() + 1) * 2 + 1) * 2);

        self.asm_print("\n");

        // self.asm_print(" -- HALT -- ");

        asm_exit();

        self.tape_print(tape.len());
    }
}
