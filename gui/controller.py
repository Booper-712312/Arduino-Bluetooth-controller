import serial
from serial import SerialException
import time

BAUDIOS = 9600
VELOCIDAD_INICIAL = "5"

bluetooth = None
velocidad_actual = VELOCIDAD_INICIAL
ultimo_movimiento = None
turbo_activado = False


def conectar(puerto):
    global bluetooth
    try:
        bluetooth = serial.Serial(puerto, BAUDIOS)
        time.sleep(1)
        bluetooth.write(VELOCIDAD_INICIAL.encode())
        return (True, f"Conexión establecida correctamente en {puerto}.")
    except SerialException as e:
        return (False, f"No fue posible conectar. Error: {e}")


def enviar(comando):
    global ultimo_movimiento, bluetooth
    if bluetooth is None:
        return ""
    if comando == ultimo_movimiento:
        return ""

    try:
        bluetooth.write(comando.encode())
        ultimo_movimiento = comando

        mensajes = {
            "F": "↑ Adelante",
            "B": "↓ Atrás",
            "L": "← Izquierda",
            "R": "→ Derecha",
            "S": "■ STOP"
        }
        if comando in mensajes:
            return f"[MOVIMIENTO] {mensajes[comando]}"
        return ""
    except SerialException:
        return "Se perdió la conexión Bluetooth."


def cambiar_velocidad(tecla):
    global velocidad_actual, bluetooth
    if bluetooth is None:
        return ""

    if velocidad_actual != tecla:
        velocidad_actual = tecla
        try:
            bluetooth.write(tecla.encode())
            return f"[VELOCIDAD] Nivel {tecla}"
        except SerialException:
            return "Error al enviar velocidad."
    return ""


def turbo(activar):
    global turbo_activado, bluetooth
    if bluetooth is None:
        return ""

    if activar and not turbo_activado:
        try:
            bluetooth.write(b'q')
            turbo_activado = True
            return "[MODO] TURBO ACTIVADO"
        except:
            pass
    elif not activar and turbo_activado:
        turbo_activado = False
    return ""


def desconectar():
    global bluetooth, ultimo_movimiento
    if bluetooth is not None:
        try:
            bluetooth.write(b'S')
            bluetooth.close()
        except:
            pass
        bluetooth = None
        ultimo_movimiento = None
        return "Puerto Bluetooth cerrado. Hasta luego."
    return ""
