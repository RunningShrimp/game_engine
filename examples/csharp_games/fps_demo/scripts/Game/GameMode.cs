using GameEngine;
using GameEngine.ECS;
using System.Collections;
using System.Collections.Generic;

namespace Game
{
    /// <summary>
    /// 游戏模式 - 管理游戏流程和规则
    /// </summary>
    public class GameMode : Component
    {
        [Header("Game Settings")]
        public int MaxPlayers = 10;
        public int GameDuration = 600; // 10分钟
        public int ScoreLimit = 50;

        [Header("Spawn Settings")]
        public float SpawnInterval = 5.0f;
        public int MaxEnemies = 20;

        [Header("Score Settings")]
        public int KillScore = 100;
        public int HeadshotScore = 150;

        // 游戏状态枚举
        public enum GameState
        {
            WaitingToStart,
            InProgress,
            GameOver
        }

        // 当前游戏状态
        public GameState State { get; private set; }
        public float GameTime { get; private set; }
        public int PlayerScore { get; private set; }
        public int EnemyCount { get; private set; }

        // 内部状态
        private float spawnTimer;
        private List<PlayerController> players = new List<PlayerController>();
        private List<Enemy> enemies = new List<Enemy>();
        private SpawnPoint[] spawnPoints;

        // 单例
        public static GameMode Instance { get; private set; }

        private void Awake()
        {
            // 设置单例
            if (Instance != null && Instance != this)
            {
                Destroy(gameObject);
                return;
            }
            Instance = this;
        }

        private void Start()
        {
            State = GameState.WaitingToStart;

            // 查找所有生成点
            spawnPoints = FindObjectsOfType<SpawnPoint>();

            // 等待玩家
            StartCoroutine(StartGameWhenReady());
        }

        private void Update(float deltaTime)
        {
            if (State == GameState.InProgress)
            {
                UpdateGame(deltaTime);
            }
        }

        private IEnumerator StartGameWhenReady()
        {
            UI.ShowMessage("等待玩家...");

            // 等待至少1个玩家
            while (players.Count == 0)
            {
                // 检查玩家是否加入
                PlayerController[] allPlayers = FindObjectsOfType<PlayerController>();
                if (allPlayers.Length > 0)
                {
                    players.AddRange(allPlayers);
                    break;
                }

                yield return new WaitForSeconds(1.0f);
            }

            // 额外等待时间
            yield return new WaitForSeconds(2.0f);

            StartGame();
        }

        public void StartGame()
        {
            State = GameState.InProgress;

            // 重置游戏
            GameTime = 0f;
            spawnTimer = 0f;
            PlayerScore = 0;

            // 清理现有敌人
            foreach (Enemy enemy in enemies)
            {
                if (enemy != null)
                {
                    Destroy(enemy.gameObject);
                }
            }
            enemies.Clear();

            // 生成初始敌人
            for (int i = 0; i < 5; i++)
            {
                SpawnEnemy();
            }

            // 显示游戏开始消息
            UI.ShowMessage("FIGHT!");

            // 播放开始音效
            AudioSource audioSource = GetComponent<AudioSource>();
            if (audioSource != null)
            {
                audioSource.Play();
            }

            log_info("游戏开始");
        }

        private void UpdateGame(float deltaTime)
        {
            GameTime += deltaTime;

            // 检查游戏结束条件
            if (GameTime >= GameDuration)
            {
                EndGame(GameOverReason.TimeUp);
                return;
            }

            if (PlayerScore >= ScoreLimit)
            {
                EndGame(GameOverReason.ScoreLimit);
                return;
            }

            // 生成敌人
            spawnTimer += deltaTime;
            if (spawnTimer >= SpawnInterval && EnemyCount < MaxEnemies)
            {
                SpawnEnemy();
                spawnTimer = 0f;
            }

            // 清理已销毁的敌人
            enemies.RemoveAll(e => e == null);
            EnemyCount = enemies.Count;

            // 更新UI
            UI.UpdateTime(GameDuration - GameTime);
            UI.UpdateScore(PlayerScore, EnemyCount);
        }

        private void SpawnEnemy()
        {
            // 从生成点选择
            Transform spawnPoint = GetRandomSpawnPoint();

            if (spawnPoint == null)
            {
                log_warning("没有可用的生成点");
                return;
            }

            // 创建敌人
            GameObject enemy = Instantiate(
                EnemyPrefab,
                spawnPoint.position,
                spawnPoint.rotation
            );

            Enemy enemyComponent = enemy.GetComponent<Enemy>();
            if (enemyComponent != null)
            {
                enemies.Add(enemyComponent);
                EnemyCount = enemies.Count;
            }

            log_info("生成敌人: " + enemy.name);
        }

        private Transform GetRandomSpawnPoint()
        {
            if (spawnPoints == null || spawnPoints.Length == 0)
            {
                return null;
            }

            // 过滤出可用的生成点
            List<SpawnPoint> availablePoints = new List<SpawnPoint>();
            foreach (SpawnPoint point in spawnPoints)
            {
                if (point != null && point.IsAvailable)
                {
                    availablePoints.Add(point);
                }
            }

            if (availablePoints.Count == 0)
            {
                return null;
            }

            // 随机选择
            int index = Random.Range(0, availablePoints.Count);
            return availablePoints[index].transform;
        }

        public void OnPlayerJoined(PlayerController player)
        {
            if (!players.Contains(player))
            {
                players.Add(player);
                log_info("玩家加入: " + player.name);
            }
        }

        public void OnPlayerKilled(PlayerController player)
        {
            players.Remove(player);

            log_info("玩家被击败: " + player.name);

            // 检查是否所有玩家都被消灭
            if (players.Count == 0)
            {
                EndGame(GameOverReason.AllPlayersDead);
            }
        }

        public void OnEnemyKilled(Enemy enemy)
        {
            enemies.Remove(enemy);

            // 增加分数
            PlayerController killer = enemy.LastHitBy;
            int score = killer != null ? KillScore : KillScore / 2;
            AddScore(score);

            log_info("敌人被击败: +{0}分", score);
        }

        public void OnEnemyKilled(Enemy enemy, bool isHeadshot)
        {
            enemies.Remove(enemy);

            // 增加分数（爆头奖励）
            PlayerController killer = enemy.LastHitBy;
            int score = killer != null ?
                (isHeadshot ? HeadshotScore : KillScore) :
                (isHeadshot ? HeadshotScore / 2 : KillScore / 2);

            AddScore(score);

            // 显示爆头提示
            if (isHeadshot)
            {
                UI.ShowHeadshot();
            }

            log_info("敌人被击败{0}: +{1}分", isHeadshot ? " (爆头)" : "", score);
        }

        public void AddScore(int points)
        {
            PlayerScore += points;

            // 检查分数限制
            if (PlayerScore >= ScoreLimit)
            {
                EndGame(GameOverReason.ScoreLimit);
            }
        }

        private enum GameOverReason
        {
            TimeUp,
            ScoreLimit,
            AllPlayersDead
        }

        private void EndGame(GameOverReason reason)
        {
            if (State == GameState.GameOver)
            {
                return;
            }

            State = GameState.GameOver;

            // 显示结果
            string reasonMessage = GetGameOverReasonMessage(reason);
            UI.ShowGameOver(PlayerScore, reasonMessage);

            // 播放游戏结束音效
            AudioSource audioSource = GetComponent<AudioSource>();
            if (audioSource != null)
            {
                audioSource.Play();
            }

            log_info("游戏结束: {0}, 最终得分: {1}", reasonMessage, PlayerScore);

            // 重启游戏
            StartCoroutine(RestartGameDelayed());
        }

        private string GetGameOverReasonMessage(GameOverReason reason)
        {
            switch (reason)
            {
                case GameOverReason.TimeUp:
                    return "时间到";
                case GameOverReason.ScoreLimit:
                    return "达到分数限制";
                case GameOverReason.AllPlayersDead:
                    return "所有玩家被击败";
                default:
                    return "游戏结束";
            }
        }

        private IEnumerator RestartGameDelayed()
        {
            yield return new WaitForSeconds(5.0f);

            // 清理场景
            CleanupGame();

            // 重新开始
            StartGame();
        }

        private void CleanupGame()
        {
            // 清理敌人
            foreach (Enemy enemy in enemies)
            {
                if (enemy != null)
                {
                    Destroy(enemy.gameObject);
                }
            }
            enemies.Clear();
            EnemyCount = 0;

            // 重置玩家
            foreach (PlayerController player in FindObjectsOfType<PlayerController>())
            {
                Health health = player.GetComponent<Health>();
                if (health != null)
                {
                    health.SetHealth(health.MaxHealth);
                }

                player.Transform.position = Vector3.Zero;
            }

            players.Clear();
        }

        public void PauseGame()
        {
            if (State == GameState.InProgress)
            {
                Time.timeScale = 0.0f;
                UI.ShowPauseMenu();
                log_info("游戏暂停");
            }
        }

        public void ResumeGame()
        {
            Time.timeScale = 1.0f;
            UI.HidePauseMenu();
            log_info("游戏继续");
        }

        public void QuitGame()
        {
            log_info("退出游戏");
            Application.Quit();
        }

        private void OnDestroy()
        {
            // 清理单例
            if (Instance == this)
            {
                Instance = null;
            }
        }

        // 日志辅助方法
        private void log_info(string message)
        {
            Debug.Log("[GameMode] " + message);
        }

        private void log_info(string format, params object[] args)
        {
            Debug.Log("[GameMode] " + string.Format(format, args));
        }

        private void log_warning(string message)
        {
            Debug.LogWarning("[GameMode] " + message);
        }

        // 敌人预制体引用
        private GameObject EnemyPrefab
        {
            get
            {
                // 从资源管理器加载敌人预制体
                // 简化示例，返回null
                return null;
            }
        }
    }
}
