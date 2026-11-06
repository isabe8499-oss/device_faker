# Teste manual da identidade no Android

Este roteiro valida somente o campo legado `android.os.Build.SERIAL` no processo do aplicativo configurado. Não valida nem promete alterar `Build.getSerial()`, `Settings.Secure.ANDROID_ID`, telephony, GAID, MediaDrm, MAC ou atestação.

## 1. Configuração de teste

Faça backup do arquivo de configuração antes de editar. Use um aplicativo de teste controlado por você e substitua o pacote abaixo pelo pacote real:

```toml
[[apps]]
package = "com.example.identityprobe"

[apps.identity]
mode = "static"
serial = "DF-TEST-ANDROID12-001"
```

O bloco de identidade é selecionado atomicamente. Se já existir uma entrada direta `[[apps]]` para esse pacote, inclua `identity` nessa mesma entrada; não crie uma segunda entrada duplicada. Se o pacote vier de um template, o bloco do template pode ser usado desde que nenhuma entrada direta substitua o registro.

Reinicie o processo do aplicativo depois de alterar a configuração. O valor é aplicado durante `pre_app_specialize`; processos já em execução não são atualizados retroativamente.

## 2. Código de leitura no aplicativo de teste

Em um aplicativo Android de teste, leia o campo estático e, separadamente, tente consultar o getter moderno:

```java
import android.os.Build;
import android.util.Log;

public final class IdentityProbe {
    private static final String TAG = "IdentityProbe";

    public static void report() {
        Log.i(TAG, "Build.SERIAL=" + Build.SERIAL);

        try {
            Log.i(TAG, "Build.getSerial()=" + Build.getSerial());
        } catch (SecurityException denied) {
            Log.i(TAG, "Build.getSerial()=SecurityException");
        } catch (Throwable error) {
            Log.w(TAG, "Build.getSerial() unavailable", error);
        }
    }
}
```

Chame `IdentityProbe.report()` de uma Activity ou de um ponto de entrada controlado pelo aplicativo.

## 3. Verificação

1. Confirme que o módulo está ativo e que o pacote de teste está incluído na configuração.
2. Force-stop o aplicativo e inicie-o novamente.
3. Leia os registros com `adb logcat -s IdentityProbe:I`.
4. No processo configurado, o campo `Build.SERIAL` deve mostrar `DF-TEST-ANDROID12-001`.
5. Em um segundo aplicativo não configurado, o campo deve permanecer no valor original da plataforma.
6. `Build.getSerial()` deve continuar sujeito às permissões e regras do Android; em um app comum, uma `SecurityException` é um resultado esperado.
7. Remova o bloco `identity`, force-stop e reinicie o app. O campo deve voltar ao comportamento original da plataforma.

## 4. Interpretação e limites

- O resultado comprova somente a leitura do campo estático legado no processo de teste.
- Não use um resultado de `Build.SERIAL` como evidência de alteração do serial físico.
- Não compare o campo legado com `Build.getSerial()` como se fossem a mesma API.
- Não use aplicativos de terceiros ou jogos como único instrumento de teste: prefira um probe mínimo e controlado.
- Faça primeiro o teste em um único app de teste. Se o campo não mudar, colete o log do módulo e o `logcat` antes de ampliar a configuração.
- A mudança é process-local; não altera `system_server`, propriedades globais nem dados assinados pelo hardware.

## 5. Matriz de validação

Registre o resultado real em cada ambiente, sem extrapolar sucesso entre versões ou fabricantes:

| Ambiente | Resultado a registrar |
|---|---|
| Motorola One Deen, Android 12 / API 31 | `Build.SERIAL`, `Build.getSerial()`, app controle, logs |
| Android 10 / API 29 | Mesmo conjunto, se disponível |
| Android 13 / API 33 | Mesmo conjunto, se disponível |
| Android 14 / API 34 | Mesmo conjunto, se disponível |
| Android 15 / API 35 | Mesmo conjunto, se disponível |

Nenhuma compatibilidade em aparelho físico é declarada até que os testes sejam executados e os resultados registrados.
