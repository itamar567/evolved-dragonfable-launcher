var receiver = new LocalConnection();
var sender = new LocalConnection();
receiver.allowDomain = function()
{
   return true;
};
receiver.swapGameLog = function(text)
{
   var lv = new LoadVars();
   var result = new LoadVars();
   lv.text = text;
   lv.sendAndLoad("http://127.0.0.1:39621/game_log", result, "POST");
};
receiver.swapBattleLog = function(text)
{
   var lv = new LoadVars();
   var result = new LoadVars();
   lv.text = text;
   lv.sendAndLoad("http://127.0.0.1:39621/battle_log", result, "POST");
};
receiver.resetLogs = function()
{
   var lv = new LoadVars();
   var result = new LoadVars();
   lv.sendAndLoad("http://127.0.0.1:39621/log_reset", result, "POST");
   sender.send("df_main", "doAfterLoad");
};
receiver.logSwap = function()
{
   var lv = new LoadVars();
   var result = new LoadVars();
   lv.sendAndLoad("http://127.0.0.1:39621/log_swap", result, "POST");
};
receiver.connect("df_log");
stop();
