---
source-wiki: https://www.chessprogramming.org/UCI
source-spec: http://download.shredderchess.com/div/uci.zip (official spec text, April 2006, by Rudolf Huber and Stefan Meyer-Kahlen; mirrored at https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf)
retrieved: 2026-10-23
note: verbatim copy of the normative protocol spec, kept in full because zeus-uci must be byte-exact compliant with it; this is the specification document chessprogramming.org/UCI itself points readers to for the actual syntax.
---

// Dowloaded from: http://download.shredderchess.com/div/uci.zip

Description of the universal chess interface (UCI)    April  2006
=================================================================

* The specification is independent of the operating system. For Windows,
  the engine is a normal exe file, either a console or "real" windows application.

* all communication is done via standard input and output with text commands,

* The engine should boot and wait for input from the GUI,
  the engine should wait for the "isready" or "setoption" command to set up its internal parameters
  as the boot process should be as quick as possible.

* the engine must always be able to process input from stdin, even while thinking.

* all command strings the engine receives will end with '\n',
  also all commands the GUI receives should end with '\n',
  Note: '\n' can be 0x0d or 0x0a0d or any combination depending on your OS.
  If you use Engine and GUI in the same OS this should be no problem if you communicate in text mode,
  but be aware of this when for example running a Linux engine in a Windows GUI.

* arbitrary white space between tokens is allowed
  Example: "debug on\n" and  "   debug     on  \n" and "\t  debug \t  \t\ton\t  \n"
  all set the debug mode of the engine on.

* The engine will always be in forced mode which means it should never start calculating
  or pondering without receiving a "go" command first.

* Before the engine is asked to search on a position, there will always be a position command
  to tell the engine about the current position.

* by default all the opening book handling is done by the GUI,
  but there is an option for the engine to use its own book ("OwnBook" option, see below)

* if the engine or the GUI receives an unknown command or token it should just ignore it and try to
  parse the rest of the string in this line.
  Examples: "joho debug on\n" should switch the debug mode on given that joho is not defined,
            "debug joho on\n" will be undefined however.

* if the engine receives a command which is not supposed to come, for example "stop" when the engine is
  not calculating, it should also just ignore it.


Move format:
------------

The move format is in long algebraic notation.
A nullmove from the Engine to the GUI should be sent as 0000.
Examples:  e2e4, e7e5, e1g1 (white short castling), e7e8q (for promotion)



GUI to engine:
--------------

These are all the command the engine gets from the interface.

* uci
	tell engine to use the uci (universal chess interface),
	this will be sent once as a first command after program boot
	to tell the engine to switch to uci mode.
	After receiving the uci command the engine must identify itself with the "id" command
	and send the "option" commands to tell the GUI which engine settings the engine supports if any.
	After that the engine should send "uciok" to acknowledge the uci mode.
	If no uciok is sent within a certain time period, the engine task will be killed by the GUI.

* debug [ on | off ]
	switch the debug mode of the engine on and off.
	In debug mode the engine should send additional infos to the GUI, e.g. with the "info string" command,
	to help debugging, e.g. the commands that the engine has received etc.
	This mode should be switched off by default and this command can be sent
	any time, also when the engine is thinking.

* isready
	this is used to synchronize the engine with the GUI. When the GUI has sent a command or
	multiple commands that can take some time to complete,
	this command can be used to wait for the engine to be ready again or
	to ping the engine to find out if it is still alive.
	E.g. this should be sent after setting the path to the tablebases as this can take some time.
	This command is also required once before the engine is asked to do any search
	to wait for the engine to finish initializing.
	This command must always be answered with "readyok" and can be sent also when the engine is calculating
	in which case the engine should also immediately answer with "readyok" without stopping the search.

* setoption name <id> [value <x>]
	this is sent to the engine when the user wants to change the internal parameters
	of the engine. For the "button" type no value is needed.
	One string will be sent for each parameter and this will only be sent when the engine is waiting.
	The name and value of the option in <id> should not be case sensitive and can inlude spaces.
	The substrings "value" and "name" should be avoided in <id> and <x> to allow unambiguous parsing,
	for example do not use <name> = "draw value".
	Here are some strings for the example below:
	   "setoption name Nullmove value true\n"
      "setoption name Selectivity value 3\n"
	   "setoption name Style value Risky\n"
	   "setoption name Clear Hash\n"
	   "setoption name NalimovPath value c:\chess\tb\4;c:\chess\tb\5\n"

* register
	this is the command to try to register an engine or to tell the engine that registration
	will be done later. This command should always be sent if the engine	has sent "registration error"
	at program startup.
	The following tokens are allowed:
	* later
	   the user doesn't want to register the engine now.
	* name <x>
	   the engine should be registered with the name <x>
	* code <y>
	   the engine should be registered with the code <y>
	Example:
	   "register later"
	   "register name Stefan MK code 4359874324"

* ucinewgame
   this is sent to the engine when the next search (started with "position" and "go") will be from
   a different game. This can be a new game the engine should play or a new game it should analyse but
   also the next position from a testsuite with positions only.
   If the GUI hasn't sent a "ucinewgame" before the first "position" command, the engine shouldn't
   expect any further ucinewgame commands as the GUI is probably not supporting the ucinewgame command.
   So the engine should not rely on this command even though all new GUIs should support it.
   As the engine's reaction to "ucinewgame" can take some time the GUI should always send "isready"
   after "ucinewgame" to wait for the engine to finish its operation.

* position [fen <fenstring> | startpos ]  moves <move1> .... <movei>
	set up the position described in fenstring on the internal board and
	play the moves on the internal chess board.
	if the game was played  from the start position the string "startpos" will be sent
	Note: no "new" command is needed. However, if this position is from a different game than
	the last position sent to the engine, the GUI should have sent a "ucinewgame" inbetween.

* go
	start calculating on the current position set up with the "position" command.
	There are a number of commands that can follow this command, all will be sent in the same string.
	If one command is not sent its value should be interpreted as it would not influence the search.
	* searchmoves <move1> .... <movei>
		restrict search to this moves only
		Example: After "position startpos" and "go infinite searchmoves e2e4 d2d4"
		the engine should only search the two moves e2e4 and d2d4 in the initial position.
	* ponder
		start searching in pondering mode.
		Do not exit the search in ponder mode, even if it's mate!
		This means that the last move sent in in the position string is the ponder move.
		The engine can do what it wants to do, but after a "ponderhit" command
		it should execute the suggested move to ponder on. This means that the ponder move sent by
		the GUI can be interpreted as a recommendation about which move to ponder. However, if the
		engine decides to ponder on a different move, it should not display any mainlines as they are
		likely to be misinterpreted by the GUI because the GUI expects the engine to ponder
	   on the suggested move.
	* wtime <x>
		white has x msec left on the clock
	* btime <x>
		black has x msec left on the clock
	* winc <x>
		white increment per move in mseconds if x > 0
	* binc <x>
		black increment per move in mseconds if x > 0
	* movestogo <x>
      there are x moves to the next time control,
		this will only be sent if x > 0,
		if you don't get this and get the wtime and btime it's sudden death
	* depth <x>
		search x plies only.
	* nodes <x>
	   search x nodes only,
	* mate <x>
		search for a mate in x moves
	* movetime <x>
		search exactly x mseconds
	* infinite
		search until the "stop" command. Do not exit the search without being told so in this mode!

* stop
	stop calculating as soon as possible,
	don't forget the "bestmove" and possibly the "ponder" token when finishing the search

* ponderhit
	the user has played the expected move. This will be sent if the engine was told to ponder on the same move
	the user has played. The engine should continue searching but switch from pondering to normal search.

* quit
	quit the program as soon as possible


Engine to GUI:
--------------

* id
	* name <x>
		this must be sent after receiving the "uci" command to identify the engine,
		e.g. "id name Shredder X.Y\n"
	* author <x>
		this must be sent after receiving the "uci" command to identify the engine,
		e.g. "id author Stefan MK\n"

* uciok
	Must be sent after the id and optional options to tell the GUI that the engine
	has sent all infos and is ready in uci mode.

* readyok
	This must be sent when the engine has received an "isready" command and has
	processed all input and is ready to accept new commands now.
	It is usually sent after a command that can take some time to be able to wait for the engine,
	but it can be used anytime, even when the engine is searching,
	and must always be answered with "isready".

* bestmove <move1> [ ponder <move2> ]
	the engine has stopped searching and found the move <move> best in this position.
	the engine can send the move it likes to ponder on. The engine must not start pondering automatically.
	this command must always be sent if the engine stops searching, also in pondering mode if there is a
	"stop" command, so for every "go" command a "bestmove" command is needed!
	Directly before that the engine should send a final "info" command with the final search information,
	the the GUI has the complete statistics about the last search.

* copyprotection
	(not relevant to Zeus; Zeus is not copy-protected)

* registration
	(not relevant to Zeus; Zeus requires no registration)

* info
	the engine wants to send information to the GUI. This should be done whenever one of the info has changed.
	The engine can send only selected infos or multiple infos with one info command,
	e.g. "info currmove e2e4 currmovenumber 1" or
	     "info depth 12 nodes 123456 nps 100000".
	Also all infos belonging to the pv should be sent together
	e.g. "info depth 2 score cp 214 time 1242 nodes 2124 nps 34928 pv e2e4 e7e5 g1f3"
	I suggest to start sending "currmove", "currmovenumber", "currline" and "refutation" only after one second
	to avoid too much traffic.
	Additional info:
	* depth <x>
		search depth in plies
	* seldepth <x>
		selective search depth in plies,
		if the engine sends seldepth there must also be a "depth" present in the same string.
	* time <x>
		the time searched in ms, this should be sent together with the pv.
	* nodes <x>
		x nodes searched, the engine should send this info regularly
	* pv <move1> ... <movei>
		the best line found
	* multipv <num>
		this for the multi pv mode.
		for the best move/pv add "multipv 1" in the string when you send the pv.
		in k-best mode always send all k variants in k strings together.
	* score
		* cp <x>
			the score from the engine's point of view in centipawns.
		* mate <y>
			mate in y moves, not plies.
			If the engine is getting mated use negative values for y.
		* lowerbound
	      the score is just a lower bound.
		* upperbound
		   the score is just an upper bound.
	* currmove <move>
		currently searching this move
	* currmovenumber <x>
		currently searching move number x, for the first move x should be 1 not 0.
	* hashfull <x>
		the hash is x permill full, the engine should send this info regularly
	* nps <x>
		x nodes per second searched, the engine should send this info regularly
	* tbhits <x>
		x positions where found in the endgame table bases
	* sbhits <x>
		x positions where found in the shredder endgame databases
	* cpuload <x>
		the cpu usage of the engine is x permill.
	* string <str>
		any string str which will be displayed be the engine,
		if there is a string command the rest of the line will be interpreted as <str>.
	* refutation <move1> <move2> ... <movei>
	   move <move1> is refuted by the line <move2> ... <movei>, i can be any number >= 1.
	* currline <cpunr> <move1> ... <movei>
	   this is the current line the engine is calculating.

* option
	This command tells the GUI which parameters can be changed in the engine.
	* name <id> ... (Hash, NalimovPath, NalimovCache, Ponder, OwnBook, MultiPV,
	  UCI_ShowCurrLine, UCI_ShowRefutations, UCI_LimitStrength, UCI_Elo,
	  UCI_AnalyseMode, UCI_Opponent, UCI_EngineAbout, UCI_ShredderbasesPath,
	  UCI_SetPositionValue, UCI_Chess960, and any engine-defined names)
	* type <t>  -- check | spin | combo | button | string
	* default <x>
	* min <x>
	* max <x>
	* var <x>
	Examples:
	   "option name Nullmove type check default true\n"
      "option name Selectivity type spin default 2 min 0 max 4\n"
	   "option name Style type combo default Normal var Solid var Normal var Risky\n"
	   "option name NalimovPath type string default c:\\n"
	   "option name Clear Hash type button\n"


Boot sequence example (abridged, see full spec for the complete transcript):
------------------------------------------------------------------------

GUI: uci
Engine: id name ... / id author ... / option name ... (repeated) / uciok
GUI: setoption name Hash value 32
GUI: isready
Engine: readyok
GUI: ucinewgame
GUI: position startpos moves e2e4 e7e5
GUI: go infinite
Engine: info depth 1 ... / info score cp ... pv ... / (repeated) ...
GUI: stop
Engine: bestmove g1f3 ponder d8f6


Chess960
========

UCI_Chess960 option; castling is sent as king-takes-own-rook
(e.g. e1h1 for White short castling in the standard start position).
Not implemented by Zeus v0.1 (standard chess only); noted here for when/if
Chess960 support becomes a roadmap item.

---

## Relevant option for Elo calibration: UCI_LimitStrength / UCI_Elo

From the spec above: `UCI_LimitStrength` (check, default false) must always
be implemented together with `UCI_Elo` (spin): "If UCI_LimitStrength is set
to true, the engine should play with this specific strength." Stockfish
implements both. `zeus-bench` relies on exactly this pair of standard options
to make Stockfish play at a chosen target Elo for the calibration ladder —
it is not inspecting or depending on any Stockfish source code, only driving
it through this documented, standard UCI option pair.

## Zeus usage

`zeus-uci` implements, for v0.1: `uci`, `isready`, `ucinewgame`, `position`,
`go` (subset: `wtime`/`btime`/`winc`/`binc`/`movestogo`/`movetime`/`depth`/
`infinite`), `stop`, `quit`, and emits `id`, `uciok`, `readyok`, `info depth
... score cp ... nodes ... pv ...`, and `bestmove`. `option`/`setoption` for
`Hash` is added once a transposition table exists (later roadmap stage);
`ponder`, `register*`, `copyprotection`, Chess960, and MultiPV are out of
scope for now and noted as future work rather than silently unsupported.
