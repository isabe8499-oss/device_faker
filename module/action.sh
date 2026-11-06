#!/system/bin/sh
# Read-only diagnostics for Device Faker. This script never changes module or Zygisk state.

MODDIR=${0%/*}
MODULE_PROP="$MODDIR/module.prop"
LIB64="$MODDIR/zygisk/arm64-v8a.so"
LIB32="$MODDIR/zygisk/armeabi-v7a.so"

print_line() {
    printf '%s\n' "$1"
}

read_prop() {
    key="$1"
    if [ -f "$MODULE_PROP" ]; then
        sed -n "s/^$key=//p" "$MODULE_PROP" | head -n 1
    fi
}

print_line "Device Faker — diagnóstico somente leitura"
print_line "------------------------------------------"
print_line "Módulo: $(read_prop name)"
print_line "Versão: $(read_prop version)"
print_line "ID: $(read_prop id)"
print_line "Android API: $(getprop ro.build.version.sdk 2>/dev/null)"
print_line "ABI principal: $(getprop ro.product.cpu.abi 2>/dev/null)"
print_line "ABI 64-bit: $(getprop ro.product.cpu.abi 2>/dev/null | grep -q 64 && printf '64-bit' || printf 'não identificada')"

if [ -f "$MODDIR/disable" ]; then
    print_line "Estado do módulo: DESATIVADO (marcador disable presente)"
elif [ -f "$MODDIR/remove" ]; then
    print_line "Estado do módulo: REMOÇÃO PENDENTE"
else
    print_line "Estado do módulo: sem marcador disable/remove"
fi

if [ -f "$MODDIR/zygisk/unloaded" ]; then
    print_line "Estado Zygisk: arquivo unloaded presente (biblioteca marcada como incompatível)"
else
    print_line "Estado Zygisk: nenhum marcador unloaded"
fi

if [ -f "$LIB64" ]; then
    print_line "Biblioteca ARM64: presente"
else
    print_line "Biblioteca ARM64: AUSENTE"
fi

if [ -f "$LIB32" ]; then
    print_line "Biblioteca ARM32: presente"
else
    print_line "Biblioteca ARM32: ausente"
fi

if [ "$ZYGISK_ENABLED" = "1" ]; then
    print_line "Variável ZYGISK_ENABLED: 1 (Zygisk reportado pelo ambiente Magisk)"
elif [ -n "$ZYGISK_ENABLED" ]; then
    print_line "Variável ZYGISK_ENABLED: $ZYGISK_ENABLED"
else
    print_line "Variável ZYGISK_ENABLED: não definida neste contexto (não prova que Zygisk esteja desligado)"
fi

print_line "------------------------------------------"
print_line "Este relatório verifica arquivos e marcadores; não confirma injeção nem execução do código nativo."
print_line "Para confirmar execução, consulte os logs do Device Faker após iniciar um app de teste controlado."
