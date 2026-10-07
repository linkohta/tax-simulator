package io.github.masterlink.taxsimulator

import android.content.res.Configuration
import android.os.Bundle
import android.view.View
import androidx.activity.enableEdgeToEdge
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    applySystemBarColors()

    // edge-to-edge 表示ではステータスバー・ナビゲーションバー・キーボードが WebView に重なる。
    // WebView の CSS（safe-area-inset）では端末によって取れないため、ネイティブ側で余白を取る。
    // キーボード（ime）も含めることで、入力中の欄が隠れないようにする。
    val content = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val bars = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or
          WindowInsetsCompat.Type.displayCutout() or
          WindowInsetsCompat.Type.ime()
      )
      view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      WindowInsetsCompat.CONSUMED
    }
  }

  // マニフェストの configChanges に uiMode があるので、ダークモードを切り替えても Activity は作り直されない
  // （作り直すと WebView が再読み込みされ、入力内容が消える）。システムバーの色はここで塗り直す。
  override fun onConfigurationChanged(newConfig: Configuration) {
    super.onConfigurationChanged(newConfig)
    applySystemBarColors()
  }

  /** 余白部分（システムバーの裏）を画面の背景色に合わせ、バーのアイコンの明暗を現在のテーマに合わせる */
  private fun applySystemBarColors() {
    window.decorView.setBackgroundColor(ContextCompat.getColor(this, R.color.system_bar_background))
    enableEdgeToEdge()
  }
}
