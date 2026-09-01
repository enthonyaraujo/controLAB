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
import java.io.FileOutputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;

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
                contentValues.put(MediaStore.Images.Media.RELATIVE_PATH, Environment.DIRECTORY_PICTURES + "/LGR Studio");
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
                File lgrDir = new File(picturesDir, "LGR Studio");
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
                contentValues.put(MediaStore.Downloads.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS + "/LGR Studio");
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
                File lgrDir = new File(downloadsDir, "LGR Studio");
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
}
