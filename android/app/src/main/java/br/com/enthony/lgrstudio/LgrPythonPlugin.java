package br.com.enthony.lgrstudio;

import com.chaquo.python.PyObject;
import com.chaquo.python.Python;
import com.getcapacitor.JSObject;
import com.getcapacitor.Plugin;
import com.getcapacitor.PluginCall;
import com.getcapacitor.PluginMethod;
import com.getcapacitor.annotation.CapacitorPlugin;

@CapacitorPlugin(name = "LgrPython")
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
}
