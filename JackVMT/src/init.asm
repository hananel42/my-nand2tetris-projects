@256
D=A
@SP
M=D




(END)
@END
0;JMP

(DO_RETURN)
//endFrame=LCL
@LCL
D=M
@R13
M=D

//retAddr=*(endFrame-5)
@5
A=D-A
D=M
@R14
M=D

//*ARG=pop
@SP
AM=M-1
D=M
@ARG
A=M
M=D

//SP=ARG+1
@ARG
D=M
@SP
M=D+1

//THAT
@R13
AM=M-1
D=M
@THAT
M=D

//THIS
@R13
AM=M-1
D=M
@THIS
M=D

//ARG
@R13
AM=M-1
D=M
@ARG
M=D

//LCL
@R13
AM=M-1
D=M
@LCL
M=D

//goto retAddr
@R14
A=M
0;JMP

(DO_CALL) //R13 - retAddr, R14 - (args+5),  R15 - function
//push ret addr
@R13
D=M
@SP
M=M+1
A=M-1
M=D

//push lcl
@LCL
D=M
@SP
M=M+1
A=M-1
M=D

//push arg
@ARG
D=M
@SP
M=M+1
A=M-1
M=D

//push this
@THIS
D=M
@SP
M=M+1
A=M-1
M=D

//push that
@THAT
D=M
@SP
M=M+1
A=M-1
M=D

//set arg
@R14
D=M
@SP
D=M-D
@ARG
M=D

//set lcl
@SP
D=M
@LCL
M=D

//jump
@R15
A=M
0;JMP
