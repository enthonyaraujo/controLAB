package br.com.enthony.lgrstudio;

import android.Manifest;
import android.content.ContentResolver;
import android.content.ContentValues;
import android.content.Context;
import android.media.MediaScannerConnection;
import android.net.Uri;
import android.os.Build;
import android.os.Environment;
import android.provider.MediaStore;
import android.util.Base64;
import com.chaquo.python.PyObject;
import com.chaquo.python.Python;
import com.getcapacitor.JSObject;
import com.getcapacitor.PermissionState;
import com.getcapacitor.Plugin;
import com.getcapacitor.PluginCall;
import com.getcapacitor.PluginMethod;
import com.getcapacitor.annotation.CapacitorPlugin;
import com.getcapacitor.annotation.Permission;
import com.getcapacitor.annotation.PermissionCallback;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import android.content.Intent;
import androidx.core.content.FileProvider;

@CapacitorPlugin(
    name = "LgrPython",
    permissions = {
        @Permission(
            alias = "storage",
            strings = {
                Manifest.permission.READ_EXTERNAL_STORAGE,
                Manifest.permission.WRITE_EXTERNAL_STORAGE
            }
        )
    }
)
public class LgrPythonPlugin extends Plugin {

    @PluginMethod
    public void dispatch(PluginCall call) {
        JSObject payload = call.getObject("payload", new JSObject());

        try {
            PyObject module = Python.getInstance().getModule("android_bridge");
            PyObject result = module.callAttr("dispatch_json", payload.toString());
            call.resolve(new JSObject(result.toString()));
        } catch (Exception error) {
            call.reject("Falha no motor Python do LGR", error);
        }
    }

    @PluginMethod
    public void saveImage(PluginCall call) {
        if (Build.VERSION.SDK_INT <= Build.VERSION_CODES.P) {
            if (getPermissionState("storage") != PermissionState.GRANTED) {
                requestPermissionForAlias("storage", call, "saveImageCallback");
                return;
            }
        }
        executeSaveImage(call);
    }

    @PermissionCallback
    private void saveImageCallback(PluginCall call) {
        if (getPermissionState("storage") == PermissionState.GRANTED) {
            executeSaveImage(call);
        } else {
            call.reject("Permissão de armazenamento negada pelo usuário.");
        }
    }

    private void executeSaveImage(PluginCall call) {
        String base64 = call.getString("base64", "");
        String defaultName = call.getString("defaultName", "lgr_grafico_" + System.currentTimeMillis() + ".png");

        if (base64 == null || base64.isEmpty()) {
            call.reject("Nenhum dado de imagem fornecido.");
            return;
        }

        if (base64.contains(",")) {
            base64 = base64.substring(base64.indexOf(",") + 1);
        }

        try {
            byte[] imageBytes = Base64.decode(base64, Base64.DEFAULT);
            Context context = getContext();

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                ContentResolver resolver = context.getContentResolver();
                ContentValues contentValues = new ContentValues();
                contentValues.put(MediaStore.Images.Media.DISPLAY_NAME, defaultName);
                contentValues.put(MediaStore.Images.Media.MIME_TYPE, "image/png");
                contentValues.put(MediaStore.Images.Media.RELATIVE_PATH, Environment.DIRECTORY_PICTURES + "/ControLAB");
                contentValues.put(MediaStore.Images.Media.IS_PENDING, 1);

                Uri uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, contentValues);
                if (uri != null) {
                    try (OutputStream outputStream = resolver.openOutputStream(uri)) {
                        if (outputStream != null) {
                            outputStream.write(imageBytes);
                        }
                    }
                    contentValues.clear();
                    contentValues.put(MediaStore.Images.Media.IS_PENDING, 0);
                    resolver.update(uri, contentValues, null, null);

                    JSObject ret = new JSObject();
                    ret.put("success", true);
                    ret.put("message", "Imagem salva com sucesso na galeria!");
                    call.resolve(ret);
                    return;
                }
            } else {
                File picturesDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_PICTURES);
                File lgrDir = new File(picturesDir, "ControLAB");
                if (!lgrDir.exists()) {
                    lgrDir.mkdirs();
                }
                File imageFile = new File(lgrDir, defaultName);
                try (FileOutputStream fos = new FileOutputStream(imageFile)) {
                    fos.write(imageBytes);
                }
                MediaScannerConnection.scanFile(context, new String[]{imageFile.getAbsolutePath()}, new String[]{"image/png"}, null);

                JSObject ret = new JSObject();
                ret.put("success", true);
                ret.put("filePath", imageFile.getAbsolutePath());
                ret.put("message", "Imagem salva na Galeria!");
                call.resolve(ret);
                return;
            }
            call.reject("Não foi possível salvar a imagem no armazenamento.");
        } catch (Exception e) {
            call.reject("Erro ao salvar imagem: " + e.getMessage(), e);
        }
    }

    @PluginMethod
    public void saveSVG(PluginCall call) {
        if (Build.VERSION.SDK_INT <= Build.VERSION_CODES.P) {
            if (getPermissionState("storage") != PermissionState.GRANTED) {
                requestPermissionForAlias("storage", call, "saveSVGCallback");
                return;
            }
        }
        executeSaveSVG(call);
    }

    @PermissionCallback
    private void saveSVGCallback(PluginCall call) {
        if (getPermissionState("storage") == PermissionState.GRANTED) {
            executeSaveSVG(call);
        } else {
            call.reject("Permissão de armazenamento negada pelo usuário.");
        }
    }

    private void executeSaveSVG(PluginCall call) {
        String svg = call.getString("svg", "");
        String defaultName = call.getString("defaultName", "lgr_grafico_" + System.currentTimeMillis() + ".svg");

        if (svg == null || svg.isEmpty()) {
            call.reject("Nenhum dado SVG fornecido.");
            return;
        }

        try {
            Context context = getContext();
            byte[] svgBytes = svg.getBytes(StandardCharsets.UTF_8);

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                ContentResolver resolver = context.getContentResolver();
                ContentValues contentValues = new ContentValues();
                contentValues.put(MediaStore.Downloads.DISPLAY_NAME, defaultName);
                contentValues.put(MediaStore.Downloads.MIME_TYPE, "image/svg+xml");
                contentValues.put(MediaStore.Downloads.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS + "/ControLAB");
                contentValues.put(MediaStore.Downloads.IS_PENDING, 1);

                Uri uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, contentValues);
                if (uri != null) {
                    try (OutputStream outputStream = resolver.openOutputStream(uri)) {
                        if (outputStream != null) {
                            outputStream.write(svgBytes);
                        }
                    }
                    contentValues.clear();
                    contentValues.put(MediaStore.Downloads.IS_PENDING, 0);
                    resolver.update(uri, contentValues, null, null);

                    JSObject ret = new JSObject();
                    ret.put("success", true);
                    ret.put("message", "Arquivo SVG salvo na pasta Downloads!");
                    call.resolve(ret);
                    return;
                }
            } else {
                File downloadsDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS);
                File lgrDir = new File(downloadsDir, "ControLAB");
                if (!lgrDir.exists()) {
                    lgrDir.mkdirs();
                }
                File svgFile = new File(lgrDir, defaultName);
                try (FileOutputStream fos = new FileOutputStream(svgFile)) {
                    fos.write(svgBytes);
                }
                MediaScannerConnection.scanFile(context, new String[]{svgFile.getAbsolutePath()}, new String[]{"image/svg+xml"}, null);

                JSObject ret = new JSObject();
                ret.put("success", true);
                ret.put("filePath", svgFile.getAbsolutePath());
                ret.put("message", "Arquivo SVG salvo na pasta Downloads!");
                call.resolve(ret);
                return;
            }
            call.reject("Não foi possível salvar o arquivo SVG.");
        } catch (Exception e) {
            call.reject("Erro ao salvar SVG: " + e.getMessage(), e);
        }
    }

    private final ExecutorService downloadExecutor = Executors.newSingleThreadExecutor();
    private volatile boolean isDownloadCanceled = false;
    private volatile HttpURLConnection activeDownloadConnection = null;

    @PluginMethod
    public void downloadUpdatePackage(PluginCall call) {
        String urlString = call.getString("url");
        String filename = call.getString("filename", "ControLAB-update.apk");
        long expectedSize = call.getLong("expectedSize", 0L);
        String token = call.getString("token", "");

        if (urlString == null || urlString.isEmpty()) {
            call.reject("URL de download ausente.");
            return;
        }

        isDownloadCanceled = false;

        downloadExecutor.execute(() -> {
            File tempDir = new File(getContext().getCacheDir(), "controlab-updates");
            if (!tempDir.exists()) {
                tempDir.mkdirs();
            }
            File outputFile = new File(tempDir, filename);
            File partFile = new File(tempDir, filename + ".part");

            InputStream in = null;
            FileOutputStream out = null;
            HttpURLConnection connection = null;

            try {
                String targetUrl = urlString;
                int redirectCount = 0;
                while (redirectCount < 8) {
                    URL u = new URL(targetUrl);
                    connection = (HttpURLConnection) u.openConnection();
                    connection.setInstanceFollowRedirects(true);
                    connection.setConnectTimeout(20000);
                    connection.setReadTimeout(30000);
                    connection.setRequestProperty("User-Agent", "ControLAB-Android-Updater");
                    if (token != null && !token.trim().isEmpty() && targetUrl.contains("api.github.com")) {
                        connection.setRequestProperty("Authorization", "Bearer " + token.trim());
                    }
                    connection.connect();
                    int status = connection.getResponseCode();
                    if (status == HttpURLConnection.HTTP_MOVED_TEMP ||
                        status == HttpURLConnection.HTTP_MOVED_PERM ||
                        status == HttpURLConnection.HTTP_SEE_OTHER ||
                        status == 307 || status == 308) {
                        targetUrl = connection.getHeaderField("Location");
                        connection.disconnect();
                        redirectCount++;
                        continue;
                    }
                    if (status != HttpURLConnection.HTTP_OK) {
                        call.reject("Servidor respondeu com codigo HTTP " + status);
                        return;
                    }
                    break;
                }

                activeDownloadConnection = connection;
                long totalBytes = connection.getContentLengthLong();
                if (totalBytes <= 0 && expectedSize > 0) {
                    totalBytes = expectedSize;
                }

                in = connection.getInputStream();
                out = new FileOutputStream(partFile);

                byte[] buffer = new byte[32768];
                long transferred = 0;
                int bytesRead;
                long lastEmitTime = System.currentTimeMillis();
                long lastTransferred = 0;

                while ((bytesRead = in.read(buffer)) != -1) {
                    if (isDownloadCanceled) {
                        break;
                    }
                    out.write(buffer, 0, bytesRead);
                    transferred += bytesRead;

                    long now = System.currentTimeMillis();
                    if (now - lastEmitTime >= 120) {
                        double deltaSec = Math.max(0.001, (now - lastEmitTime) / 1000.0);
                        long bytesPerSec = (long) ((transferred - lastTransferred) / deltaSec);
                        int percent = totalBytes > 0 ? (int) Math.min(100, (transferred * 100) / totalBytes) : 0;

                        JSObject prog = new JSObject();
                        prog.put("percent", percent);
                        prog.put("transferred", transferred);
                        prog.put("total", totalBytes);
                        prog.put("bytesPerSecond", bytesPerSec);
                        notifyListeners("updateProgress", prog);

                        lastEmitTime = now;
                        lastTransferred = transferred;
                    }
                }

                out.flush();
                out.close();
                out = null;
                in.close();
                in = null;

                if (isDownloadCanceled) {
                    if (partFile.exists()) partFile.delete();
                    JSObject ret = new JSObject();
                    ret.put("success", false);
                    ret.put("canceled", true);
                    call.resolve(ret);
                    return;
                }

                if (outputFile.exists()) {
                    outputFile.delete();
                }
                if (!partFile.renameTo(outputFile)) {
                    try (InputStream fis = new FileInputStream(partFile);
                         FileOutputStream fos = new FileOutputStream(outputFile)) {
                        byte[] buf = new byte[16384];
                        int r;
                        while ((r = fis.read(buf)) != -1) {
                            fos.write(buf, 0, r);
                        }
                    }
                    partFile.delete();
                }

                JSObject finalProg = new JSObject();
                finalProg.put("percent", 100);
                finalProg.put("transferred", transferred);
                finalProg.put("total", totalBytes > 0 ? totalBytes : transferred);
                finalProg.put("bytesPerSecond", 0);
                notifyListeners("updateProgress", finalProg);

                JSObject ret = new JSObject();
                ret.put("success", true);
                ret.put("filePath", outputFile.getAbsolutePath());
                call.resolve(ret);
            } catch (Exception e) {
                if (partFile.exists()) partFile.delete();
                if (isDownloadCanceled) {
                    JSObject ret = new JSObject();
                    ret.put("success", false);
                    ret.put("canceled", true);
                    call.resolve(ret);
                } else {
                    call.reject("Erro no download da atualizacao: " + e.getMessage(), e);
                }
            } finally {
                activeDownloadConnection = null;
                try { if (in != null) in.close(); } catch (Exception ignored) {}
                try { if (out != null) out.close(); } catch (Exception ignored) {}
            }
        });
    }

    @PluginMethod
    public void cancelUpdateDownload(PluginCall call) {
        isDownloadCanceled = true;
        if (activeDownloadConnection != null) {
            new Thread(() -> {
                try {
                    activeDownloadConnection.disconnect();
                } catch (Exception ignored) {}
            }).start();
        }
        JSObject ret = new JSObject();
        ret.put("success", true);
        call.resolve(ret);
    }

    @PluginMethod
    public void installUpdatePackage(PluginCall call) {
        String filePath = call.getString("filePath");
        if (filePath == null || filePath.isEmpty()) {
            call.reject("Caminho do arquivo nao fornecido.");
            return;
        }

        File apkFile = new File(filePath);
        if (!apkFile.exists()) {
            call.reject("Arquivo APK nao encontrado no dispositivo.");
            return;
        }

        try {
            Context context = getContext();
            Uri contentUri = FileProvider.getUriForFile(
                context,
                context.getPackageName() + ".fileprovider",
                apkFile
            );

            Intent intent = new Intent(Intent.ACTION_VIEW);
            intent.setDataAndType(contentUri, "application/vnd.android.package-archive");
            intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);

            context.startActivity(intent);

            JSObject ret = new JSObject();
            ret.put("success", true);
            ret.put("method", "package-installer");
            call.resolve(ret);
        } catch (Exception e) {
            call.reject("Falha ao abrir instalador do Android: " + e.getMessage(), e);
        }
    }
}
