"""
====================================================
          PC Bluetooth Controller
                Version 2.0
====================================================
Control por teclado vía Bluetooth.
"""

import serial
from serial import SerialException
import keyboard
import time

PUERTO = "COM11"
BAUDIOS = 9600
VELOCIDAD_INICIAL = "5"

bluetooth = None
ultimo_mov = None
velocidad_actual = VELOCIDAD_INICIAL

front_lights = False
back_lights = False
extra = False
horn = False

def conectar():
    global bluetooth
    try:
        print("Conectando...")
        bluetooth = serial.Serial(PUERTO, BAUDIOS)
        time.sleep(3)
        bluetooth.write(VELOCIDAD_INICIAL.encode())
        print("Bluetooth conectado.")
        return True
    except SerialException as e:
        print("Error:", e)
        return False

def enviar(cmd, mostrar=True):
    global ultimo_mov
    if cmd in ("F","B","L","R","G","I","H","J","S"):
        if ultimo_mov == cmd:
            return
        ultimo_mov = cmd
    bluetooth.write(cmd.encode())
    if mostrar:
        print(">", cmd)

def toggle_front():
    global front_lights
    front_lights = not front_lights
    enviar("W" if front_lights else "w")
    print("Luces delanteras", "ON" if front_lights else "OFF")

def toggle_back():
    global back_lights
    back_lights = not back_lights
    enviar("U" if back_lights else "u")
    print("Luces traseras", "ON" if back_lights else "OFF")

def toggle_extra():
    global extra
    extra = not extra
    enviar("X" if extra else "x")
    print("Extra", "ON" if extra else "OFF")

def velocidad():
    global velocidad_actual
    if keyboard.is_pressed("\\"):
        if velocidad_actual != "|":
            velocidad_actual="|"
            bluetooth.write(b"0")
            print("Velocidad 0%")
            time.sleep(0.2)
    for i in range(10):
        if keyboard.is_pressed(str(i)):
            if velocidad_actual != str(i):
                velocidad_actual=str(i)
                bluetooth.write((b"q" if i==0 else str(i).encode()))
                print(f"Velocidad {100 if i==0 else i*10}%")
                time.sleep(0.2)

print("""
============= PC Bluetooth Controller =============
Movimiento:
 W Adelante      Q Adelante-Izq      E Adelante-Der
 S Atrás         Z Atrás-Izq         C Atrás-Der
 A Izquierda     D Derecha           X Stop

Funciones:
 I Luces delanteras
 K Luces traseras
 H Claxon
 P Extra
 \\ Velocidad 0%
 1..9 = 10..90%
 0 = 100%
 ESC = Salir
===================================================
""")

if not conectar():
    raise SystemExit

fi=bi=ei=False

try:
    while True:
        velocidad()

        if keyboard.is_pressed("i"):
            if not fi:
                toggle_front(); fi=True
        else: fi=False

        if keyboard.is_pressed("k"):
            if not bi:
                toggle_back(); bi=True
        else: bi=False

        if keyboard.is_pressed("p"):
            if not ei:
                toggle_extra(); ei=True
        else: ei=False

        if keyboard.is_pressed("h"):
            if not horn:
                horn=True
                enviar("V")
        else:
            if horn:
                horn=False
                enviar("v")

        if keyboard.is_pressed("q"):
            enviar("G")
        elif keyboard.is_pressed("e"):
            enviar("I")
        elif keyboard.is_pressed("z"):
            enviar("H")
        elif keyboard.is_pressed("c"):
            enviar("J")
        elif keyboard.is_pressed("w"):
            enviar("F")
        elif keyboard.is_pressed("s"):
            enviar("B")
        elif keyboard.is_pressed("a"):
            enviar("L")
        elif keyboard.is_pressed("d"):
            enviar("R")
        elif keyboard.is_pressed("x"):
            enviar("S")
        else:
            enviar("S", False)

        if keyboard.is_pressed("esc"):
            enviar("S")
            break

        time.sleep(0.02)

finally:
    if bluetooth:
        bluetooth.close()
    print("Programa finalizado.")
