using GameEngine;
using GameEngine.ECS;
using System.Linq;
using System.Collections.Generic;

namespace BrickBreaker
{
    /// <summary>
    /// 游戏管理器 - 控制游戏流程和状态
    /// </summary>
    public class GameManager : Component
    {
        // ========== 静态实例 ==========

        public static GameManager Instance { get; private set; }

        // ========== 公共属性 ==========

        /// <summary>当前分数</summary>
        public int Score { get; private set; }

        /// <summary>剩余生命</summary>
        public int Lives { get; private set; }

        /// <summary>当前关卡</summary>
        public int Level { get; private set; }

        /// <summary>游戏状态</summary>
        public GameState State { get; private set; }

        /// <summary>最高分数</summary>
        public int HighScore { get; private set; }

        // ========== 私有字段 ==========

        private Ball ball;
        private Paddle paddle;
        private List<Brick> bricks;

        // ========== 生命周期方法 ==========

        public void Awake()
        {
            // 设置单例
            if (Instance != null && Instance != this)
            {
                Debug.LogWarning("Multiple GameManager instances detected!");
                return;
            }
            Instance = this;

            // 初始化状态
            State =GameState.Menu;
            HighScore = PlayerPrefs.GetInt("HighScore", 0);
        }

        public void Start()
        {
            // 初始化游戏
            StartGame();
        }

        public void OnDestroy()
        {
            if (Instance == this)
            {
                Instance = null;
            }
        }

        public void Update(float deltaTime)
        {
            // 检查暂停键
            if (Input.GetKeyDown(KeyCode.Escape))
            {
                TogglePause();
            }

            // 检查R键重新开始
            if (Input.GetKeyDown(KeyCode.R) && State == GameState.GameOver)
            {
                RestartGame();
            }
        }

        // ========== 游戏流程方法 ==========

        /// <summary>开始新游戏</summary>
        public void StartGame()
        {
            Debug.Log("Starting new game...");

            Score = 0;
            Lives = 3;
            Level = 1;
            State = GameState.Playing;

            // 更新UI
            UI.UpdateScore(Score);
            UI.UpdateLives(Lives);
            UI.UpdateLevel(Level);

            // 创建游戏对象
            FindGameObjects();
            ResetGameObjects();
            CreateLevel();

            // 显示开始消息
            UI.ShowMessage("Level 1 - Press SPACE to Start!");
        }

        /// <summary>增加分数</summary>
        public void AddScore(int points)
        {
            Score += points;
            UI.UpdateScore(Score);

            // 检查是否打破最高分
            if (Score > HighScore)
            {
                HighScore = Score;
                PlayerPrefs.SetInt("HighScore", HighScore);
            }

            // 检查是否过关
            CheckLevelComplete();
        }

        /// <summary>失去一条生命</summary>
        public void LoseLife()
        {
            Lives--;
            UI.UpdateLives(Lives);

            if (Lives <= 0)
            {
                GameOver();
            }
            else
            {
                ResetBall();
                UI.ShowMessage($"Life Lost! {Lives} left. Press SPACE");
            }
        }

        /// <summary>游戏结束</summary>
        public void GameOver()
        {
            State = GameState.GameOver;
            Time.timeScale = 0f;

            UI.ShowGameOver(Score, HighScore);

            Debug.Log($"Game Over! Score: {Score}");
        }

        /// <summary>重新开始游戏</summary>
        public void RestartGame()
        {
            Time.timeScale = 1f;
            StartGame();
        }

        /// <summary>进入下一关</summary>
        public void NextLevel()
        {
            Level++;
            UI.UpdateLevel(Level);

            // 创建新关卡
            CreateLevel();

            // 重置球
            ResetBall();

            // 显示消息
            UI.ShowMessage($"Level {Level} - Press SPACE!");
        }

        /// <summary>切换暂停状态</summary>
        public void TogglePause()
        {
            if (State == GameState.Playing)
            {
                State = GameState.Paused;
                Time.timeScale = 0f;
                UI.ShowPauseMenu();
            }
            else if (State == GameState.Paused)
            {
                State = GameState.Playing;
                Time.timeScale = 1f;
                UI.HidePauseMenu();
            }
        }

        // ========== 私有方法 ==========

        private void FindGameObjects()
        {
            ball = FindObjectOfType<Ball>();
            paddle = FindObjectOfType<Paddle>();

            if (ball == null)
                Debug.LogError("Ball not found!");
            if (paddle == null)
                Debug.LogError("Paddle not found!");
        }

        private void ResetGameObjects()
        {
            // 重置球
            if (ball != null)
            {
                ball.Reset();
            }

            // 重置挡板
            if (paddle != null)
            {
                paddle.ResetSize();
                paddle.Transform.Position = new Vector3(0, -5, 0);
            }
        }

        private void ResetBall()
        {
            if (ball != null)
            {
                ball.Reset();
            }
        }

        private void CreateLevel()
        {
            // 清除旧砖块
            Brick[] oldBricks = FindObjectsOfType<Brick>();
            foreach (Brick brick in oldBricks)
            {
                World.DestroyEntity(brick.Entity);
            }

            // 根据关卡创建砖块
            int rows = Mathf.Min(3 + Level, 8);
            int cols = 10;

            bricks = new List<Brick>();

            for (int row = 0; row < rows; row++)
            {
                for (int col = 0; col < cols; col++)
                {
                    CreateBrick(row, col, cols);
                }
            }

            Debug.Log($"Created {bricks.Count} bricks for Level {Level}");
        }

        private void CreateBrick(int row, int col, int totalCols)
        {
            Entity brickEntity = World.CreateEntity();

            // 设置位置
            float x = (col - totalCols / 2.0f + 0.5f) * 1.5f;
            float y = 4.0f - row * 0.6f;
            brickEntity.Transform.Position = new Vector3(x, y, 0);

            // 添加砖块组件
            Brick brick = brickEntity.AddComponent<Brick>();

            // 设置砖块属性
            brick.Points = (rows - row) * 10;
            brick.Color = GetRowColor(row);
            brick.Hits = row < 2 ? 2 : 1; // 前两行需要打两次
            brick.Type = GetBrickType(row, Level);

            // 特殊砖块
            if (brick.Type == BrickType.Indestructible)
            {
                brick.Indestructible = true;
                brick.Points = 0;
            }

            // 添加视觉组件
            SpriteRenderer spriteRenderer = brickEntity.AddComponent<SpriteRenderer>();
            spriteRenderer.Sprite = Resources.Load<Sprite>("Bricks/Brick_" + brick.Type.ToString());
            spriteRenderer.Color = brick.Color;

            bricks.Add(brick);
        }

        private Color GetRowColor(int row)
        {
            Color[] colors = {
                new Color(1, 0, 0),       // 红
                new Color(1, 0.5f, 0),    // 橙
                new Color(1, 1, 0),       // 黄
                new Color(0, 1, 0),       // 绿
                new Color(0, 0, 1),       // 蓝
                new Color(0.5f, 0, 1),    // 紫
                new Color(1, 0, 1),       // 品红
                new Color(0.5f, 0, 0.5f)  // 棕
            };

            return colors[row % colors.Length];
        }

        private BrickType GetBrickType(int row, int level)
        {
            // 根据关卡和行数决定砖块类型
            if (level >= 3 && row == 0)
                return BrickType.Indestructible;

            if (level >= 2 && row == rows - 1)
                return BrickType.Explosive;

            float random = Random.value;
            if (random < 0.1f)
                return BrickType.PowerUp;

            return BrickType.Normal;
        }

        private void CheckLevelComplete()
        {
            if (bricks == null || bricks.Count == 0)
                return;

            // 计算剩余的可破坏砖块
            int remainingBricks = bricks.Count(b => !b.Indestructible);

            if (remainingBricks == 0)
            {
                Debug.Log("Level Complete!");
                NextLevel();
            }
        }
    }

    /// <summary>
    /// 游戏状态枚举
    /// </summary>
    public enum GameState
    {
        /// <summary>菜单</summary>
        Menu,

        /// <summary>游戏中</summary>
        Playing,

        /// <summary>暂停</summary>
        Paused,

        /// <summary>游戏结束</summary>
        GameOver
    }
}
