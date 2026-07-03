package com.example.app;

import android.app.Activity;
import android.os.Bundle;

public class MainActivity extends Activity {
    private String title;

    public MainActivity() {
        title = "Home";
    }

    @Override
    protected void onCreate(Bundle state) {
        super.onCreate(state);
    }

    interface Screen {
        void render();
    }

    enum Mode {
        Light,
        Dark
    }
}
