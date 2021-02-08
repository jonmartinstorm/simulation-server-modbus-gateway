TYPE
  LOGLEVEL : (CRITICAL, WARNING, INFO, DEBUG) := INFO;
END_TYPE

FUNCTION_BLOCK LOGGER
  VAR_INPUT
    TRIG : BOOL;
    MSG : STRING;
    LEVEL : LOGLEVEL := INFO;
  END_VAR
  VAR
    TRIG0 : BOOL;
  END_VAR

  IF TRIG AND NOT TRIG0 THEN
  {{
   LogMessage(GetFbVar(LEVEL),(char*)GetFbVar(MSG, .body),GetFbVar(MSG, .len));
  }}
  END_IF;
  TRIG0:=TRIG;
END_FUNCTION_BLOCK



FUNCTION_BLOCK python_eval
  VAR_INPUT
    TRIG : BOOL;
    CODE : STRING;
  END_VAR
  VAR_OUTPUT
    ACK : BOOL;
    RESULT : STRING;
  END_VAR
  VAR
    STATE : DWORD;
    BUFFER : STRING;
    PREBUFFER : STRING;
    TRIGM1 : BOOL;
    TRIGGED : BOOL;
  END_VAR

  {extern void __PythonEvalFB(int, PYTHON_EVAL*);__PythonEvalFB(0, data__);}
END_FUNCTION_BLOCK

FUNCTION_BLOCK python_poll
  VAR_INPUT
    TRIG : BOOL;
    CODE : STRING;
  END_VAR
  VAR_OUTPUT
    ACK : BOOL;
    RESULT : STRING;
  END_VAR
  VAR
    STATE : DWORD;
    BUFFER : STRING;
    PREBUFFER : STRING;
    TRIGM1 : BOOL;
    TRIGGED : BOOL;
  END_VAR

  {extern void __PythonEvalFB(int, PYTHON_EVAL*);__PythonEvalFB(1,(PYTHON_EVAL*)(void*)data__);}
END_FUNCTION_BLOCK

FUNCTION_BLOCK python_gear
  VAR_INPUT
    N : UINT;
    TRIG : BOOL;
    CODE : STRING;
  END_VAR
  VAR_OUTPUT
    ACK : BOOL;
    RESULT : STRING;
  END_VAR
  VAR
    py_eval : python_eval;
    COUNTER : UINT;
    ADD10_OUT : UINT;
    EQ13_OUT : BOOL;
    SEL15_OUT : UINT;
    AND7_OUT : BOOL;
  END_VAR

  ADD10_OUT := ADD(COUNTER, 1);
  EQ13_OUT := EQ(N, ADD10_OUT);
  SEL15_OUT := SEL(EQ13_OUT, ADD10_OUT, 0);
  COUNTER := SEL15_OUT;
  AND7_OUT := AND(EQ13_OUT, TRIG);
  py_eval(TRIG := AND7_OUT, CODE := CODE);
  ACK := py_eval.ACK;
  RESULT := py_eval.RESULT;
END_FUNCTION_BLOCK


PROGRAM Hello_World
  VAR
    level : UINT;
    outflow : UINT;
  END_VAR
  VAR
    max_level : REAL := 1200.0;
    min_level : REAL := 800.0;
    convert_to_level : REAL := 32.768;
    convert_from_flow : REAL := 1638.4;
    low_flow : REAL := 15.0;
    high_flow : REAL := 25.0;
    UINT_TO_REAL3_OUT : REAL;
    DIV10_OUT : REAL;
    GT11_OUT : BOOL;
    LT13_OUT : BOOL;
    SEL16_OUT : REAL;
    SEL15_OUT : REAL;
    MUL8_OUT : REAL;
    REAL_TO_UINT4_OUT : UINT;
  END_VAR

  UINT_TO_REAL3_OUT := UINT_TO_REAL(level);
  DIV10_OUT := DIV(UINT_TO_REAL3_OUT, convert_to_level);
  GT11_OUT := GT(DIV10_OUT, max_level);
  LT13_OUT := LT(DIV10_OUT, min_level);
  SEL16_OUT := SEL(LT13_OUT, SEL15_OUT, low_flow);
  SEL15_OUT := SEL(GT11_OUT, SEL16_OUT, high_flow);
  MUL8_OUT := MUL(SEL15_OUT, convert_from_flow);
  REAL_TO_UINT4_OUT := REAL_TO_UINT(MUL8_OUT);
  outflow := REAL_TO_UINT4_OUT;
END_PROGRAM


CONFIGURATION Config0

  RESOURCE Res0 ON PLC
    TASK task0(INTERVAL := T#300ms,PRIORITY := 0);
    PROGRAM instance0 WITH task0 : Hello_World;
  END_RESOURCE
END_CONFIGURATION
